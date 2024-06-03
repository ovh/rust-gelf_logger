// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

/// Logs a message with the specific level.
///
/// # Examples
///
/// ```
/// use gelf_logger::{gelf_log, GelfLevel};
///
/// gelf_log!(GelfLevel::Informational, "Something happened");
/// gelf_log!(GelfLevel::Informational, foo = "bar"; "Something happened");
/// gelf_log!(target: "app-1", GelfLevel::Informational, foo = "bar"; "Something happened");
/// ```
#[macro_export]
macro_rules! gelf_log {
    // gelf_log!(target: "my_target", GelfLevel::Informational, key1:? = 42, key2 = true; "a {} event", "log");
    (target: $target:expr, $lvl:expr, $($key:tt $(:$capture:tt)? $(= $value:expr)?),+; $($arg:tt)+) => ({
        let log_lvl = log::Level::from($lvl);
        if log_lvl <= log::STATIC_MAX_LEVEL && log_lvl <= log::max_level() {
            let lvl_key = $crate::INTERNAL_LEVEL_FIELD_NAME;
            let kvs = [(lvl_key, log::__log_value!(lvl_key = $lvl as u32)), $((log::__log_key!($key), log::__log_value!($key $(:$capture)* = $($value)*))),+];
            let mut builder = log::Record::builder();
            builder
                .args(format_args!($($arg)+))
                .level(log_lvl) // Will be overwrite.
                .target($target)
                .module_path_static(Some(module_path!()))
                .file_static(Some(file!()))
                .line(Some(line!()))
                .key_values(&kvs);
            log::logger().log(&builder.build());
        }
    });

    // gelf_log!(target: "my_target", GelfLevel::Informational, "a {} event", "log");
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
        let log_lvl = log::Level::from($lvl);
        if log_lvl <= log::STATIC_MAX_LEVEL && log_lvl <= log::max_level() {
            let kvs = [($crate::INTERNAL_LEVEL_FIELD_NAME, $lvl as u32)];
            let mut builder = log::Record::builder();
            builder
                .args(format_args!($($arg)+))
                .level(log_lvl) // Will be overwrite.
                .target($target)
                .module_path_static(Some(module_path!()))
                .file_static(Some(file!()))
                .line(Some(line!()))
                .key_values(&kvs);
            log::logger().log(&builder.build());
        }
    });

    // gelf_log!(GelfLevel::Informational, "a log event")
    ($lvl:expr, $($arg:tt)+) => ($crate::gelf_log!(target: module_path!(), $lvl, $($arg)+));
}

/// Logs a message at the emergency level (A "panic" condition).
///
/// Notify all tech staff on call? (Earthquake? Tornado?) - affects multiple
/// apps/servers/sites.
///
/// # Examples
///
/// ```
/// use gelf_logger::gelf_emergency;
///
/// gelf_emergency!("System is unusable!!");
/// gelf_emergency!(foo = "bar"; "System is unusable!!");
/// gelf_emergency!(target: "app-1", foo = "bar"; "System is unusable!!");
/// ```
#[macro_export]
macro_rules! gelf_emergency {
    (target: $target:expr, $($arg:tt)+) => ($crate::gelf_log!(target: $target, gelf_logger::GelfLevel::Emergency, $($arg)+));
    ($($arg:tt)+) => ($crate::gelf_log!(gelf_logger::GelfLevel::Emergency, $($arg)+))
}

/// Logs a message at the alert level (Should be corrected immediately).
///
/// Notify staff who can fix the problem - example is loss of backup ISP
/// connection.
///
/// # Examples
///
/// ```
/// use gelf_logger::gelf_alert;
///
/// gelf_alert!("Action must be taken immediately.");
/// gelf_alert!(foo = "bar"; "Action must be taken immediately.");
/// gelf_alert!(target: "app-1", foo = "bar"; "Action must be taken immediately.");
/// ```
#[macro_export]
macro_rules! gelf_alert {
    (target: $target:expr, $($arg:tt)+) => ($crate::gelf_log!(target: $target, gelf_logger::GelfLevel::Alert, $($arg)+));
    ($($arg:tt)+) => ($crate::gelf_log!(gelf_logger::GelfLevel::Alert, $($arg)+))
}

/// Logs a message at the critical level (Should be corrected immediately).
///
/// Should be corrected immediately, but indicates failure in a primary system -
/// fix CRITICAL problems before ALERT - example is loss of primary ISP
/// connection.
///
/// # Examples
///
/// ```
/// use gelf_logger::gelf_critical;
///
/// gelf_critical!("No space left on device");
/// gelf_critical!(foo = "bar"; "No space left on device");
/// gelf_critical!(target: "app-1", foo = "bar"; "No space left on device");
/// ```
#[macro_export]
macro_rules! gelf_critical {
    (target: $target:expr, $($arg:tt)+) => ($crate::gelf_log!(target: $target, gelf_logger::GelfLevel::Critical, $($arg)+));
    ($($arg:tt)+) => ($crate::gelf_log!(gelf_logger::GelfLevel::Critical, $($arg)+))
}

/// Logs a message at the error level (Non-urgent failures).
///
/// These should be relayed to developers or admins; each item must be resolved
/// within a given time.
///
/// # Examples
///
/// ```
/// use gelf_logger::gelf_error;
///
/// gelf_error!("Login failed!");
/// gelf_error!(foo = "bar"; "Login failed!");
/// gelf_error!(target: "app-1", foo = "bar"; "Login failed!");
/// ```
#[macro_export]
macro_rules! gelf_error {
    (target: $target:expr, $($arg:tt)+) => ($crate::gelf_log!(target: $target, gelf_logger::GelfLevel::Error, $($arg)+));
    ($($arg:tt)+) => ($crate::gelf_log!(gelf_logger::GelfLevel::Error, $($arg)+))
}

/// Logs a message at the warning level (Warning messages).
///
/// Not an error, but indication that an error will occur if action is not
/// taken, e.g. file system 85% full - each item must be resolved within a given
/// time.
///
/// # Examples
///
/// ```
/// use gelf_logger::gelf_warn;
///
/// gelf_warn!("Error while fetching metadata with correlation");
/// gelf_warn!(foo = "bar"; "Error while fetching metadata with correlation");
/// gelf_warn!(target: "app-1", foo = "bar"; "Error while fetching metadata with correlation");
/// ```
#[macro_export]
macro_rules! gelf_warn {
    (target: $target:expr, $($arg:tt)+) => ($crate::gelf_log!(target: $target, gelf_logger::GelfLevel::Warning, $($arg)+));
    ($($arg:tt)+) => ($crate::gelf_log!(gelf_logger::GelfLevel::Warning, $($arg)+))
}

/// Logs a message at the notice level (Unusual event).
///
/// Events that are unusual but not error conditions - might be summarized in an
/// email to developers or admins to spot potential problems - no immediate
/// action required.
///
/// # Examples
///
/// ```
/// use gelf_logger::gelf_notice;
///
/// gelf_notice!("User reached 90% of his quota");
/// gelf_notice!(foo = "bar"; "User reached 90% of his quota");
/// gelf_notice!(target: "app-1", foo = "bar"; "User reached 90% of his quota");
/// ```
#[macro_export]
macro_rules! gelf_notice {
    (target: $target:expr, $($arg:tt)+) => ($crate::gelf_log!(target: $target, gelf_logger::GelfLevel::Notice, $($arg)+));
    ($($arg:tt)+) => ($crate::gelf_log!(gelf_logger::GelfLevel::Notice, $($arg)+))
}

/// Logs a message at the info level (Normal message).
///
/// Normal operational messages - may be harvested for reporting, measuring
/// throughput, etc.
/// - no action required.
///
/// # Examples
///
/// ```
/// use gelf_logger::gelf_info;
///
/// gelf_info!("Downloading file...");
/// gelf_info!(foo = "bar"; "Downloading file...");
/// gelf_info!(target: "app-1", foo = "bar"; "Downloading file...");
/// ```
#[macro_export]
macro_rules! gelf_info {
    (target: $target:expr, $($arg:tt)+) => ($crate::gelf_log!(target: $target, gelf_logger::GelfLevel::Informational, $($arg)+));
    ($($arg:tt)+) => ($crate::gelf_log!(gelf_logger::GelfLevel::Informational, $($arg)+))
}

/// Logs a message at the debug level (Mainly used by developers).
///
/// Info useful to developers for debugging the app, not useful during
/// operations.
///
/// # Examples
///
/// ```
/// use gelf_logger::gelf_debug;
///
/// gelf_debug!("Some debug data");
/// gelf_debug!(foo = "bar"; "Some debug data");
/// gelf_debug!(target: "app-1", foo = "bar"; "Some debug data");
/// ```
#[macro_export]
macro_rules! gelf_debug {
    (target: $target:expr, $($arg:tt)+) => ($crate::gelf_log!(target: $target, gelf_logger::GelfLevel::Debugging, $($arg)+));
    ($($arg:tt)+) => ($crate::gelf_log!(gelf_logger::GelfLevel::Debugging, $($arg)+))
}
