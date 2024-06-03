// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

use log::Level;
use serde::{Deserialize, Serialize};

/// An enum representing the record level which is equal to the standard syslog
/// levels.
#[derive(Serialize, Deserialize, PartialOrd, PartialEq, Copy, Clone, Debug)]
pub enum GelfLevel {
    /// The "Emergency" level.
    ///
    /// System is unusable.
    Emergency = 0,
    /// The "Alert" level.
    ///
    /// Action must be taken immediately.
    Alert = 1,
    /// The "Critical" level.
    ///
    /// Critical conditions such as Hard device, memory errors...
    Critical = 2,
    /// The "Error" level.
    ///
    /// Error conditions.
    Error = 3,
    /// The "Warning" level.
    ///
    /// Warning conditions.
    Warning = 4,
    /// The "Notice" level.
    ///
    /// Normal but significant conditions. Conditions that are not error
    /// conditions, but that may require special handling.
    Notice = 5,
    /// The "Informational" level.
    ///
    /// Informational messages.
    Informational = 6,
    /// The "Debugging" level.
    ///
    /// Debug-level messages. Messages that contain information normally of use
    /// only when debugging a program.
    Debugging = 7,
}

/// Set the default level to `GelfLevel::Alert` according to the [specification](https://archivedocs.graylog.org/en/latest/pages/gelf.html).
impl Default for GelfLevel {
    fn default() -> GelfLevel {
        GelfLevel::Alert
    }
}

impl From<Level> for GelfLevel {
    fn from(level: Level) -> GelfLevel {
        match level {
            Level::Error => GelfLevel::Error,
            Level::Warn => GelfLevel::Warning,
            Level::Info => GelfLevel::Informational,
            Level::Debug => GelfLevel::Debugging,
            Level::Trace => GelfLevel::Debugging,
        }
    }
}

impl From<GelfLevel> for Level {
    fn from(level: GelfLevel) -> Self {
        match level {
            GelfLevel::Emergency => Level::Error,
            GelfLevel::Alert => Level::Error,
            GelfLevel::Critical => Level::Error,
            GelfLevel::Error => Level::Error,
            GelfLevel::Warning => Level::Warn,
            GelfLevel::Notice => Level::Info,
            GelfLevel::Informational => Level::Info,
            GelfLevel::Debugging => Level::Debug,
        }
    }
}

impl From<u32> for GelfLevel {
    fn from(level: u32) -> Self {
        match level {
            0 => GelfLevel::Emergency,
            1 => GelfLevel::Alert,
            2 => GelfLevel::Critical,
            3 => GelfLevel::Error,
            4 => GelfLevel::Warning,
            5 => GelfLevel::Notice,
            6 => GelfLevel::Informational,
            7 => GelfLevel::Debugging,
            _ => GelfLevel::Alert,
        }
    }
}

impl From<GelfLevel> for &'static str {
    fn from(level: GelfLevel) -> Self {
        match level {
            GelfLevel::Emergency => "Emergency",
            GelfLevel::Alert => "Alert",
            GelfLevel::Critical => "Critical",
            GelfLevel::Error => "Error",
            GelfLevel::Warning => "Warning",
            GelfLevel::Notice => "Notice",
            GelfLevel::Informational => "Informational",
            GelfLevel::Debugging => "Debugging",
        }
    }
}
