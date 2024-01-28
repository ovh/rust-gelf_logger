// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

use std::{
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread,
    time::Duration,
};

use serde_gelf::{GelfLevel, GelfRecord, GelfRecordGetter};

use crate::{
    buffer::{Buffer, Event},
    config::{Config, FullBufferPolicy},
    logger::GelfLogger,
    output::GelfTcpOutput,
    result::Result,
};

static mut BATCH_PROCESSOR: &'static dyn Batch = &NoProcessor;

pub(crate) fn set_boxed_processor(processor: Box<dyn Batch>) -> Result<()> {
    set_processor_inner(|| unsafe { &*Box::into_raw(processor) })
}

fn set_processor_inner<F>(make_processor: F) -> Result<()>
where
    F: FnOnce() -> &'static dyn Batch,
{
    unsafe {
        BATCH_PROCESSOR = make_processor();
        Ok(())
    }
}

/// Initialize the logger using a configuration file.
///
/// ### Warning
///
/// The logging system may only be initialized once.
///
/// ## Example
///
/// ```no_run
/// use gelf_logger::Config;
/// use log::info;
///
/// let cfg = Config::try_from_yaml("/tmp/myconfig.yml").unwrap();
/// gelf_logger::init(cfg).unwrap();
///
/// info!("hello");
///
/// gelf_logger::flush().expect("Failed to send buffer, log records can be lost !");
/// ```
#[cfg(feature = "yaml")]
pub fn init_from_file(path: &str) -> Result<()> {
    init(Config::try_from_yaml(path)?)
}

/// Initialize the logger using the given [`Config`](struct.Config.html).
///
/// ### Warning
///
/// The logging system may only be initialized once.
///
/// ## Example
///
/// ```rust
/// use gelf_logger::Config;
/// use log::info;
///
/// let cfg = Config::builder()
///     .set_hostname("myhost.com".into())
///     .set_port(12202)
///     .build();
///
/// gelf_logger::init(cfg).unwrap();
///
/// info!("hello");
///
/// gelf_logger::flush().expect("Failed to send buffer, log records can be lost!");
/// ```
pub fn init(cfg: Config) -> Result<()> {
    let processor = init_processor(&cfg)?;

    let log_level = log::Level::from(&cfg.level());
    let logger = GelfLogger::new(log_level);

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(log_level.to_level_filter());

    set_boxed_processor(Box::new(processor))?;

    Ok(())
}

/// Initialize the BatchProcessor.
pub fn init_processor(cfg: &Config) -> Result<BatchProcessor> {
    let (tx, rx): (SyncSender<Event>, Receiver<Event>) =
        sync_channel(cfg.async_buffer_size().unwrap_or(1000));

    let config = cfg.clone();

    thread::spawn(move || {
        let gelf_tcp_output = GelfTcpOutput::from(&config);
        Buffer::new(
            rx,
            gelf_tcp_output,
            config.buffer_size(),
            config.buffer_duration(),
        )
        .run();
    });

    let gelf_level = cfg.level();

    Ok(BatchProcessor::new(
        tx,
        gelf_level,
        cfg.full_buffer_policy()
            .unwrap_or(FullBufferPolicy::Discard),
    ))
}

/// Force current logger record buffer to be sent to the remote server.
///
/// It can be useful to perform a flush just before program exit.
///
/// ## Example
///
/// ```rust
/// use gelf_logger::Config;
/// use log::info;
///
/// let cfg = Config::builder()
///     .set_hostname("myhost.com".into())
///     .set_port(12202)
///     .build();
///
/// gelf_logger::init(cfg).unwrap();
///
/// info!("hello");
///
/// gelf_logger::flush().expect("Failed to send buffer, log records can be lost !");
/// ```
pub fn flush() -> Result<()> {
    processor().flush()
}

/// Trait for async batch processing of `GelfRecord`.
pub trait Batch {
    /// Send the `GelfRecord` in the async batch processor
    ///
    /// Records will actually be sent depending on configuration options
    fn send(&self, rec: &GelfRecord) -> Result<()>;
    /// Flushes buffered records to the network
    fn flush(&self) -> Result<()>;
}

pub(crate) struct NoProcessor;

impl Batch for NoProcessor {
    fn send(&self, _rec: &GelfRecord) -> Result<()> {
        Ok(())
    }
    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

/// Struct to send event in thread
pub struct BatchProcessor {
    tx: SyncSender<Event>,
    level: GelfLevel,
    full_buffer_policy: FullBufferPolicy,
}

impl BatchProcessor {
    /// Create a ne processor
    pub fn new(
        tx: SyncSender<Event>,
        level: GelfLevel,
        full_buffer_policy: FullBufferPolicy,
    ) -> BatchProcessor {
        BatchProcessor {
            tx,
            level,
            full_buffer_policy,
        }
    }
}

impl Batch for BatchProcessor {
    fn send(&self, rec: &GelfRecord) -> Result<()> {
        if self.level >= rec.level() {
            match self.full_buffer_policy {
                FullBufferPolicy::Wait => self.tx.send(Event::Data(rec.clone()))?,
                FullBufferPolicy::Discard => self.tx.try_send(Event::Data(rec.clone()))?,
            }
        }
        Ok(())
    }
    fn flush(&self) -> Result<()> {
        self.tx.send(Event::Flush)?;
        // FIXME it would be nice to have something more deterministic
        thread::sleep(Duration::from_secs(2));
        Ok(())
    }
}

/// Returns a reference to the batch processor.
///
/// If a logger has not been set, a no-op implementation is returned.
pub fn processor() -> &'static dyn Batch {
    unsafe { BATCH_PROCESSOR }
}
