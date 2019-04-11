use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::thread;

use serde_gelf::level::GelfLevel;
use serde_gelf::record::{GelfRecord, GelfRecordGetter};

use crate::buffer::{Buffer, Event, Metronome};
use crate::config::{Config, ConfigGetters};
use crate::logger::GelfLogger;
use crate::output::GelfTcpOutput;
use crate::result::Result;
use std::time::Duration;

static mut BATCH_PROCESSOR: &'static Batch = &NoProcessor;

pub fn set_boxed_processor(processor: Box<Batch>) -> Result<()> {
    set_processor_inner(|| unsafe { &*Box::into_raw(processor) })
}

fn set_processor_inner<F>(make_processor: F) -> Result<()> where F: FnOnce() -> &'static Batch {
    unsafe {
        BATCH_PROCESSOR = make_processor();
        Ok(())
    }
}

pub fn init_from_file(path: &str) -> Result<()> {
    init(Config::try_from_yaml(path)?)
}

pub fn init(cfg: Config) -> Result<()> {
    let (tx, rx): (SyncSender<Event>, Receiver<Event>) = sync_channel(10_000_000);

    if let &Some(duration) = cfg.buffer_duration() {
        let ctx = tx.clone();
        Metronome::new(duration).start(ctx);
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

pub trait Batch {
    fn send(&self, rec: &GelfRecord) -> Result<()>;
}


pub struct NoProcessor;

impl Batch for NoProcessor {
    fn send(&self, _rec: &GelfRecord) -> Result<()> { Ok(()) }
}


pub struct BatchProcessor {
    tx: SyncSender<Event>,
    level: GelfLevel,
}

impl BatchProcessor {
    pub fn new(tx: SyncSender<Event>, level: GelfLevel) -> BatchProcessor {
        BatchProcessor { tx, level }
    }
    pub fn flush(&self) -> Result<()> {
        let _ = self.tx.send(Event::Send)?;
        Ok(thread::sleep(Duration::from_secs(2)))
    }
}

impl Batch for BatchProcessor {
    fn send(&self, rec: &GelfRecord) -> Result<()> {
        if self.level >= rec.level() {
            self.tx.send(Event::Data(rec.clone()))?;
        }
        Ok(())
    }
}

impl Drop for BatchProcessor {
    fn drop(&mut self) {
        println!("Exiting, purging buffer...");
        let _ = self.flush();
    }
}

pub fn processor() -> &'static Batch { unsafe { BATCH_PROCESSOR } }