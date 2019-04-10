#[macro_export]
macro_rules! gelf_log {
    (level: $level:expr, extra: $extra:expr, $($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::from($level), extra: $extra, $($arg)+));
    );
    (level: $level:expr, $($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::from($level), extra: BTreeMap::new(), $($arg)+));
    );
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::default(), extra: $extra, $($arg)+));
    );
    ($($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::default(), extra: BTreeMap::new(), $($arg)+));
    );
}

#[macro_export]
macro_rules! gelf_emergency {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Emergency, extra: $extra, $($arg)+));
    );
    ($($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Emergency, extra: BTreeMap::new(), $($arg)+));
    );
}

#[macro_export]
macro_rules! gelf_alert {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Alert, extra: $extra, $($arg)+));
    );
    ($($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Alert, extra: BTreeMap::new(), $($arg)+));
    );
}

#[macro_export]
macro_rules! gelf_critical {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Critical, extra: $extra, $($arg)+));
    );
    ($($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Critical, extra: BTreeMap::new(), $($arg)+));
    );
}

#[macro_export]
macro_rules! gelf_error {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Error, extra: $extra, $($arg)+));
    );
    ($($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Error, extra: BTreeMap::new(), $($arg)+));
    );
}

#[macro_export]
macro_rules! gelf_warn {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Warning, extra: $extra, $($arg)+));
    );
    ($($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Warning, extra: BTreeMap::new(), $($arg)+));
    );
}

#[macro_export]
macro_rules! gelf_notice {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Notice, extra: $extra, $($arg)+));
    );
    ($($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Notice, extra: BTreeMap::new(), $($arg)+));
    );
}

#[macro_export]
macro_rules! gelf_info {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Informational, extra: $extra, $($arg)+));
    );
    ($($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Informational, extra: BTreeMap::new(), $($arg)+));
    );
}

#[macro_export]
macro_rules! gelf_debug {
    (extra: $extra:expr, $($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Debugging, extra: $extra, $($arg)+));
    );
    ($($arg:tt)+) => (
        $crate::processor().send(&gelf_record!(level: serde_gelf::level::GelfLevel::Debugging, extra: BTreeMap::new(), $($arg)+));
    );
}
