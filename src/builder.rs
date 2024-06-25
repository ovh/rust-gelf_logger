// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

use std::{env, time::Duration};

use env_filter::Builder as FilterBuilder;
use log::LevelFilter;

use crate::{
    error::Error,
    logger::{GelfLogger, Target, TcpTarget, Writer},
    record::flatten,
    Map, Value,
};

const DEFAULT_FILTER_ENV: &str = "RUST_LOG";

/// A [`GelfLogger`] builder.
///
/// # Examples
///
/// ```rust,no_run
/// use gelf_logger::Builder;
/// use log::LevelFilter;
///
/// // Will print GELF record to stderr.
/// Builder::new().stderr().init();
///
/// // Will send Gelf records into a background thread then to localhost over TCP.
/// Builder::new()
///     .filter_level(LevelFilter::Info)
///     .hostname("127.0.0.1".to_owned())
///     .port(2202)
///     .tls(false)
///     .init();
/// ```
#[derive(Debug)]
pub struct Builder {
    filter: FilterBuilder,
    target: Target,
    null_character: bool,
    type_suffix: bool,
    additional_fields: Map<String, Value>,
    raw_additional_fields: Map<String, Value>,
}

impl Builder {
    /// Crate a new Builder.
    ///
    /// This is equivalent to calling [`Builder::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes the log builder from the environment using default variable
    /// name (`RUST_LOG`).
    pub fn from_default_env() -> Self {
        Self::new().parse_default_env()
    }

    /// Initializes the log builder from the environment.
    ///
    /// The variables used to read configuration from can be tweaked before
    /// passing in.
    pub fn from_env(env: &str) -> Self {
        Self::new().parse_env(env)
    }

    /// Applies the configuration from the environment using default variable
    /// name (`RUST_LOG`).
    pub fn parse_default_env(self) -> Self {
        self.parse_env(DEFAULT_FILTER_ENV)
    }

    /// Applies the configuration from the environment.
    ///
    /// This function allows a builder to be configured with default parameters,
    /// to be then overridden by the environment.
    pub fn parse_env(self, env: &str) -> Self {
        let Ok(value) = env::var(env) else {
            return self;
        };
        self.parse_filters(&value)
    }

    /// Adds a directive to the filter for a specific module.
    pub fn filter_module(mut self, module: &str, level: LevelFilter) -> Self {
        self.filter.filter_module(module, level);
        self
    }

    /// Adds a directive to the filter for all modules.
    pub fn filter_level(mut self, level: LevelFilter) -> Self {
        self.filter.filter_level(level);
        self
    }

    /// Adds filters to the logger.
    ///
    /// The given module (if any) will log at most the specified level provided.
    /// If no module is provided then the filter will apply to all log messages.
    pub fn filter(mut self, module: Option<&str>, level: LevelFilter) -> Self {
        self.filter.filter(module, level);
        self
    }

    /// Parses the directives string in the same form as the `RUST_LOG`
    /// environment variable.
    ///
    /// See the [`logs`](https://docs.rs/log/latest/log/#)  documentation for more details.
    pub fn parse_filters(mut self, filters: &str) -> Self {
        self.filter.parse(filters);
        self
    }

    /// Overwrite the target with the specified one.
    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    /// Overwrite the target to set it to `stdout`.
    pub fn stdout(mut self) -> Self {
        self.target = Target::Stdout;
        self
    }

    /// Overwrite the target to set it to `stderr`.
    pub fn stderr(mut self) -> Self {
        self.target = Target::Stderr;
        self
    }

    /// Overwrite the target to set it to an TCP target. If `None` is specified
    /// [`TcpTarget::default`] will be used.
    pub fn tcp(mut self, config: Option<TcpTarget>) -> Self {
        self.target = Target::Tcp(config.unwrap_or_default());
        self
    }

    /// Set the TCP hostname. This hostname is also used to establish TLS
    /// connexion if the `tls` option is requested.
    ///
    /// If the target is currently not TCP, it will first set it.
    pub fn hostname(mut self, hostname: String) -> Self {
        self.tcp_config_or_default().hostname = hostname;
        self
    }

    /// Set the TCP port.
    ///
    /// If the target is currently not TCP, it will first set it.
    pub fn port(mut self, port: u16) -> Self {
        self.tcp_config_or_default().port = port;
        self
    }

    /// Enable or disable TLS support.
    ///
    /// If the target is currently not TCP, it will first set it.
    pub fn tls(mut self, tls: bool) -> Self {
        self.tcp_config_or_default().tls = tls;
        self
    }

    /// Set the connection timeout duration. If `None` is specified, the socket
    /// connection phase can block indefinitely.
    ///
    /// If the target is currently not TCP, it will first set it.
    pub fn connect_timeout(mut self, duration: Option<Duration>) -> Self {
        self.tcp_config_or_default().connect_timeout = duration;
        self
    }

    /// Set the connection write timeout duration. If `None` is specified, the
    /// socket write calls can block indefinitely.
    ///
    /// If the target is currently not TCP, it will first set it.
    pub fn write_timeout(mut self, duration: Option<Duration>) -> Self {
        self.tcp_config_or_default().write_timeout = duration;
        self
    }

    /// Set the number of messages that can be queued between the caller and
    /// background threads. If too many log calls are made and the background is
    /// too slow, this buffer will fill up. When full, calls on the current
    /// thread will start to block.
    ///
    /// If the target is currently not TCP, it will first set it.
    pub fn buffer_size(mut self, n: usize) -> Self {
        self.tcp_config_or_default().buffer_size = n;
        self
    }

    /// Register a static function that will be called when errors occur in the
    /// background thread.
    ///
    /// If the target is currently not TCP, it will first set it.
    pub fn background_error_handler(mut self, f: Option<fn(Error)>) -> Self {
        self.tcp_config_or_default().background_error_handler = f;
        self
    }

    fn tcp_config_or_default(&mut self) -> &mut TcpTarget {
        match &mut self.target {
            Target::Tcp(target) => target,
            target => {
                *target = Target::Tcp(TcpTarget::default());
                match target {
                    Target::Tcp(target) => target,
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Set up the builder to be used with OVH's LDP service over TLS.
    ///
    /// This is equivalent to the following configuration:
    /// ```rust,ignore
    /// builder.hostname(hostname)
    ///     .port(12202)
    ///     .tls(true)
    ///     .ovh_token(token)
    ///     .null_character(true)
    ///     .type_suffix(true)
    /// ```
    #[cfg(feature = "ovh-ldp")]
    pub fn ovh_ldp(self, hostname: String, token: String) -> Self {
        self.hostname(hostname)
            .port(12202)
            .tls(true)
            .ovh_token(token)
            .null_character(true)
            .type_suffix(true)
    }

    /// Enable or disable automatic null character (`\0`) appending at the end
    /// of every record. This may be required by some backends.
    pub fn null_character(mut self, enabled: bool) -> Self {
        self.null_character = enabled;
        self
    }

    /// Enable or disable automatic appending type suffix to additional fields
    /// according to this [documentation](https://help.ovhcloud.com/csm/en-logs-data-platform-field-naming-conventions?id=kb_article_view&sysparm_article=KB0055662).
    pub fn type_suffix(mut self, enabled: bool) -> Self {
        self.type_suffix = enabled;
        self
    }

    /// Add additional fields that will be flatted and added to every GELF
    /// record.
    pub fn extend_additional_fields<T: IntoIterator<Item = (String, Value)>>(
        mut self,
        fields: T,
    ) -> Self {
        self.additional_fields.extend(fields);
        self
    }

    /// Add raw additional fields that will be added to every GELF record.
    ///
    /// Certain backend may reject record with unexpect fields.
    pub fn extend_raw_additional_fields<T: IntoIterator<Item = (String, Value)>>(
        mut self,
        fields: T,
    ) -> Self {
        self.raw_additional_fields.extend(fields);
        self
    }

    /// Set the `X-OVH-TOKEN` field.
    #[cfg(feature = "ovh-ldp")]
    pub fn ovh_token(mut self, token: String) -> Self {
        _ = self
            .raw_additional_fields
            .insert("X-OVH-TOKEN".to_owned(), Value::String(token));
        self
    }

    /// Initializes the global logger with the built env logger.
    ///
    /// This should be called early in the execution of a Rust program. Any log
    /// events that occur before initialization will be ignored.
    ///
    /// # Errors
    ///
    /// This function will fail if it is called more than once, or if another
    /// library has already initialized a global logger.
    pub fn try_init(self) -> Result<(), Error> {
        let logger = self.build()?;

        let max_level = logger.filter();
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(max_level);

        Ok(())
    }

    /// Initializes the global logger with the built env logger.
    ///
    /// This should be called early in the execution of a Rust program. Any log
    /// events that occur before initialization will be ignored.
    ///
    /// # Panics
    ///
    /// This function will panic if it is called more than once, or if another
    /// library has already initialized a global logger.
    pub fn init(self) {
        self.try_init().expect("logger initialization failure");
    }

    /// Build the final `GelfLogger`.
    pub fn build(mut self) -> Result<GelfLogger, Error> {
        Ok(GelfLogger {
            filter: self.filter.build(),
            writer: Writer::new(self.target)?,
            null_character: self.null_character,
            additional_fields: flatten(self.additional_fields, Some("_"), "_", self.type_suffix)
                .into_iter()
                .chain(self.raw_additional_fields)
                .collect(),
        })
    }
}

impl Default for Builder {
    /// Creates a default builder that will log every record to `stderr`, with
    /// no additional fields and no null_character at the end.
    fn default() -> Self {
        Self {
            filter: FilterBuilder::default(),
            target: Target::Stderr,
            null_character: false,
            type_suffix: false,
            additional_fields: Map::new(),
            raw_additional_fields: Map::new(),
        }
    }
}
