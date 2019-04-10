use log::{Level, Log, Metadata, Record};
use serde_gelf::record::GelfRecord;

use crate::batch::processor;

pub struct GelfLogger {
    level: Level
}

impl Log for GelfLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }
    fn log(&self, record: &Record) {
        let _ = processor().send(&GelfRecord::from(record));
    }
    fn flush(&self) {}
}

impl GelfLogger {
    pub fn new(level: Level) -> GelfLogger {
        GelfLogger { level }
    }
}