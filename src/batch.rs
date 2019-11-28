// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2009 The gelf_logger Authors. All rights reserved.

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::thread;
use std::time::Duration;

use serde_gelf::{GelfLevel, GelfRecord, GelfRecordGetter};

use crate::buffer::{Buffer, Event, Metronome};
use crate::config::Config;
use crate::logger::GelfLogger;
use crate::output::GelfTcpOutput;
use crate::result::Result;

static mut BATCH_PROCESSOR: &'static dyn Batch = &NoProcessor;

pub fn set_boxed_processor(processor: Box<dyn Batch>) -> Result<()> {
    set_processor_inner(|| unsafe { &*Box::into_raw(processor) })
}

fn set_processor_inner<F>(make_processor: F) -> Result<()> where F: FnOnce() -> &'static dyn Batch {
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
/// ```rust
/// use gelf_logger::Config;
///
/// fn main() {
///     let cfg = Config::try_from_yaml("/tmp/myconfig.yml").unwrap();
///     gelf_logger::init(cfg).unwrap();
///
///     info!("hello");
///
///     gelf_logger::flush().expect("Failed to send buffer, log records can be lost !");
/// }
/// ```
///
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
///
/// fn main() {
///     let cfg = Config::builder()
///         .set_hostname("myhost.com".into())
///         .set_port(12202)
///         .build();
///
///     gelf_logger::init(cfg).unwrap();
///
///     info!("hello");
///
///     gelf_logger::flush().expect("Failed to send buffer, log records can be lost !");
/// }
/// ```
///
pub fn init(cfg: Config) -> Result<()> {
    let (tx, rx): (SyncSender<Event>, Receiver<Event>) = sync_channel(10_000_000);

    if let &Some(duration) = cfg.buffer_duration() {
        let ctx = tx.clone();
        Metronome::start(duration, ctx);
    }

    let gelf_level = cfg.level().clone();
    let log_level = log::Level::from(&gelf_level);

    let logger = GelfLogger::new(log_level);
    let arx = Arc::new(Mutex::new(rx));
    thread::spawn(move || {
        let _ = Buffer::new(arx, GelfTcpOutput::from(&cfg)).run();
    });

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(log_level.to_level_filter());

    let _ = set_boxed_processor(Box::new(BatchProcessor::new(tx, gelf_level)))?;

    Ok(())
}

/// Force current logger record buffer to be sent to the remote server.
///
/// It can be useful to perform a flush just before program exit.
///
/// ## Example
///
/// ```rust
/// use gelf_logger::Config;
///
/// fn main() {
///     let cfg = Config::builder()
///         .set_hostname("myhost.com".into())
///         .set_port(12202)
///         .build();
///
///     gelf_logger::init(cfg).unwrap();
///
///     info!("hello");
///
///     gelf_logger::flush().expect("Failed to send buffer, log records can be lost !");
/// }
/// ```
///
pub fn flush() -> Result<()> {
    processor().flush()
}

pub trait Batch {
    fn send(&self, rec: &GelfRecord) -> Result<()>;
    fn flush(&self) -> Result<()>;
}


pub struct NoProcessor;

impl Batch for NoProcessor {
    fn send(&self, _rec: &GelfRecord) -> Result<()> { Ok(()) }
    fn flush(&self) -> Result<()> { Ok(()) }
}


pub struct BatchProcessor {
    tx: SyncSender<Event>,
    level: GelfLevel,
}

impl BatchProcessor {
    pub fn new(tx: SyncSender<Event>, level: GelfLevel) -> BatchProcessor {
        BatchProcessor { tx, level }
    }
}

impl Batch for BatchProcessor {
    fn send(&self, rec: &GelfRecord) -> Result<()> {
        if self.level >= rec.level() {
            self.tx.send(Event::Data(rec.clone()))?;
        }
        Ok(())
    }
    fn flush(&self) -> Result<()> {
        let _ = self.tx.send(Event::Send)?;
        Ok(thread::sleep(Duration::from_secs(2)))
    }
}

/// Returns a reference to the batch processor.
///
/// If a logger has not been set, a no-op implementation is returned.
#[doc(hidden)]
pub fn processor() -> &'static dyn Batch { unsafe { BATCH_PROCESSOR } }
