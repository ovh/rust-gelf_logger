// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

/// The standard logging macro.
///
/// This macro will generically log with the specified `GelfLevel`, any struct
/// which implement `Serialize` and `format!` based argument list.
///
/// # Examples
///
/// ```rust
/// use gelf_logger::gelf_log;
/// use serde_derive::Serialize;
/// use serde_gelf::GelfLevel;
///
/// #[derive(Serialize)]
/// struct Myapp {
///    name: String,
///    version: String,
/// }
///
/// gelf_log!(level: GelfLevel::Error, "Hello!");
///
/// let myapp = Myapp { name: "myapp".into(), version: "0.0.1".into() };
/// gelf_log!(level: GelfLevel::Debugging, extra: &myapp, "myapp state");
/// ```
#[macro_export]
macro_rules! gelf_log {
    (level: $level:expr, extra: $extra:expr, $($arg:tt)+) => {
        {
            use std::collections::BTreeMap;
            use serde_gelf::GelfRecordBuilder;

            let _ = $crate::processor().send(&serde_gelf::GelfRecord::new()
                .set_facility(module_path!().to_string())
                .set_file(file!().to_string())
                .set_line(line!())
                .set_level($level)
                .set_message(format_args!($($arg)+).to_string())
                .extend_additional_fields(serde_gelf::to_flat_dict($extra).unwrap_or_default())
            );
        }
    };
    (level: $level:expr, $($arg:tt)+) => {
        gelf_log!(level: $level, extra: &std::collections::BTreeMap::<(), ()>::new(), $($arg)+);
    };
    (extra: $extra:expr, $($arg:tt)+) => {
        gelf_log!(level: serde_gelf::GelfLevel::default(), extra: $extra, $($arg)+);
    };
    ($($arg:tt)+) => {
        gelf_log!(level: serde_gelf::GelfLevel::default(), extra: &std::collections::BTreeMap::<(), ()>::new(), $($arg)+);
    };
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
/// gelf_emergency!("System is unusable !!");
/// ```
#[macro_export]
macro_rules! gelf_emergency {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Emergency, extra: $extra, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Emergency, extra: &BTreeMap::<(), ()>::new(), $($arg)+);
    );
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
/// ```
#[macro_export]
macro_rules! gelf_alert {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Alert, extra: $extra, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Alert, extra: &BTreeMap::<(), ()>::new(), $($arg)+);
    );
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
/// ```
#[macro_export]
macro_rules! gelf_critical {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Critical, extra: $extra, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Critical, extra: &BTreeMap::<(), ()>::new(), $($arg)+);
    );
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
/// ```
#[macro_export]
macro_rules! gelf_error {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Error, extra: $extra, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Error, extra: &BTreeMap::<(), ()>::new(), $($arg)+);
    );
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
/// ```
#[macro_export]
macro_rules! gelf_warn {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Warning, extra: $extra, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Warning, extra: &BTreeMap::<(), ()>::new(), $($arg)+);
    );
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
/// ```
#[macro_export]
macro_rules! gelf_notice {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Notice, extra: $extra, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Notice, extra: &BTreeMap::<(), ()>::new(), $($arg)+);
    );
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
/// ```
#[macro_export]
macro_rules! gelf_info {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Informational, extra: $extra, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Informational, extra: &BTreeMap::<(), ()>::new(), $($arg)+);
    );
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
/// gelf_debug!("myvalue = '5'");
/// ```
#[macro_export]
macro_rules! gelf_debug {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Debugging, extra: $extra, $($arg)+);
    );
    ($($arg:tt)+) => (
        $crate::gelf_log!(level: serde_gelf::GelfLevel::Debugging, extra: &BTreeMap::<(), ()>::new(), $($arg)+);
    );
}
