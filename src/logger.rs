// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

use log::{Level, Log, Metadata, Record};
use serde_gelf::GelfRecord;

use crate::batch::processor;

pub struct GelfLogger {
    level: Level,
}

impl Log for GelfLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }
    fn log(&self, record: &Record) {
        let _ = processor().send(&GelfRecord::from(record));
    }
    fn flush(&self) {
        let _ = processor().flush();
    }
}

impl GelfLogger {
    pub fn new(level: Level) -> GelfLogger {
        GelfLogger { level }
    }
}
