// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2009 The gelf_logger Authors. All rights reserved.

use std::collections::BTreeMap;
use std::fs::File;

use serde_gelf::GelfLevel;
use serde_value::Value;

use crate::result::Result;

/// Builder for [`Config`](struct.Config.html).
///
/// The ConfigBuilder can set the different parameters of Config object, and returns the created
/// object when build is called.
///
/// ## Example
///
/// ```rust
/// use gelf_logger::ConfigBuilder;
///
/// let cfg = ConfigBuilder::new()
///     .set_hostname("localhost".into())
///     .build();
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigBuilder {
    level: GelfLevel,
    hostname: String,
    port: u64,
    null_character: bool,
    use_tls: bool,
    async_buffer_size: Option<usize>,
    buffer_size: Option<usize>,
    buffer_duration: Option<u64>,
    additional_fields: BTreeMap<Value, Value>,
    full_buffer_policy: Option<FullBufferPolicy>,
    connect_timeout_ms: Option<u64>,
    write_timeout_ms: Option<u64>,
}

impl Default for ConfigBuilder {
    /// Construct default ConfigBuilder.
    ///
    /// Defaults values are:
    ///
    /// - level: `GelfLevel::Alert`
    /// - hostname: "127.0.0.1"
    /// - port: 12202
    /// - null_character: true
    /// - use_tls: true
    /// - buffer_size: None
    /// - buffer_duration: None
    /// - additional_fields: empty BTreeMap
    ///
    fn default() -> Self {
        ConfigBuilder {
            level: GelfLevel::default(),
            hostname: "127.0.0.1".to_string(),
            port: 12202,
            null_character: true,
            use_tls: true,
            async_buffer_size: None,
            buffer_size: None,
            buffer_duration: None,
            additional_fields: BTreeMap::default(),
            full_buffer_policy: Some(FullBufferPolicy::Discard),
            connect_timeout_ms: None,
            write_timeout_ms: None,
        }
    }
}

impl ConfigBuilder {
    /// Load configuration using the given `path` file.
    /// ## Example
    ///
    /// ```rust
    /// use gelf_logger::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::try_from_yaml("/tmp/myconf.yml")
    ///     .expect("Invalid config file!")
    ///     .build();
    /// ```
    pub fn try_from_yaml(path: &str) -> Result<ConfigBuilder> {
        Ok(serde_yaml::from_reader(File::open(path)?)?)
    }
    /// Sets threshold for this logger to level. Logging messages which are less severe than level
    /// will be ignored.
    pub fn set_level(mut self, level: GelfLevel) -> ConfigBuilder {
        self.level = level;
        self
    }
    /// Sets the hostname of the remote server.
    pub fn set_hostname(mut self, hostname: String) -> ConfigBuilder {
        self.hostname = hostname;
        self
    }
    /// Sets the port of the remote server.
    pub fn set_port(mut self, port: u64) -> ConfigBuilder {
        self.port = port;
        self
    }
    /// Adds a NUL byte (`\0`) after each entry.
    pub fn set_null_character(mut self, null_character: bool) -> ConfigBuilder {
        self.null_character = null_character;
        self
    }
    /// Activate transport security.
    pub fn set_use_tls(mut self, use_tls: bool) -> ConfigBuilder {
        self.use_tls = use_tls;
        self
    }
    /// Set the asynchronous buffer size. This buffer is placed between the log subsystem
    /// and the network sender. This represent the maximum number of message the system
    /// will buffer before blocking while waiting for message to be actually sent to the
    /// remote server.
    ///
    /// Default: 1000
    ///
    /// ### Warning
    ///
    /// This actually allocates a buffer of this size, if you set a high value here,
    /// is will eat a large amount of memory.
    pub fn set_async_buffer_size(mut self, async_buffer_size: usize) -> ConfigBuilder {
        self.async_buffer_size = Some(async_buffer_size);
        self
    }

    /// Sets the upperbound limit on the number of records that can be placed in the buffer, once
    /// this size has been reached, the buffer will be sent to the remote server.
    pub fn set_buffer_size(mut self, buffer_size: usize) -> ConfigBuilder {
        self.buffer_size = Some(buffer_size);
        self
    }
    /// Sets the maximum lifetime (in milli seconds) of the buffer before send it to the remote
    /// server.
    pub fn set_buffer_duration(mut self, buffer_duration: u64) -> ConfigBuilder {
        self.buffer_duration = Some(buffer_duration);
        self
    }
    /// Adds an additional data which will be append to each log entry.
    pub fn put_additional_field<V>(mut self, key: String, value: V) -> ConfigBuilder
    where
        V: Into<Value>,
    {
        self.additional_fields
            .insert(Value::String(key), value.into());
        self
    }
    /// Adds multiple additional data which will be append to each log entry.
    pub fn extend_additional_fields(
        mut self,
        additional_fields: BTreeMap<Value, Value>,
    ) -> ConfigBuilder {
        self.additional_fields.extend(additional_fields);
        self
    }
    /// Set the policy to apply when async send buffer is full.
    ///
    /// It is recommended to use the `FullBufferPolicy::Discard` policy.
    ///
    /// If not set or set to `None`, `FullBufferPolicy::Discard` will be used by default
    pub fn set_full_buffer_policy(mut self, policy: Option<FullBufferPolicy>) -> ConfigBuilder {
        self.full_buffer_policy = policy;
        self
    }
    /// Set the TCP connect timeout.    
    pub fn set_connect_timeout_ms(mut self, connect_timeout_ms: Option<u64>) -> ConfigBuilder {
        self.connect_timeout_ms = connect_timeout_ms;
        self
    }
    /// Set the TCP write timeout.    
    pub fn set_write_timeout_ms(mut self, write_timeout_ms: Option<u64>) -> ConfigBuilder {
        self.write_timeout_ms = write_timeout_ms;
        self
    }

    /// Invoke the builder and return a Config
    pub fn build(self) -> Config {
        Config {
            level: self.level,
            hostname: self.hostname,
            port: self.port,
            null_character: self.null_character,
            use_tls: self.use_tls,
            async_buffer_size: self.async_buffer_size,
            buffer_size: self.buffer_size,
            buffer_duration: self.buffer_duration,
            additional_fields: self.additional_fields,
            full_buffer_policy: self.full_buffer_policy,
            connect_timeout_ms: self.connect_timeout_ms,
            write_timeout_ms: self.write_timeout_ms,
        }
    }
}

/// The policy to apply when the async buffer is full.
///
/// This policy does not apply to `flush` methods.
#[derive(Deserialize, Debug, Clone, Copy)]
pub enum FullBufferPolicy {
    /// Wait for the log entry to be consumed by the BatchProcessor.
    ///
    /// If the async buffer is full, subsequent calls to log() will wait for
    /// space in the buffer. Note that this will bock the application. Use this
    /// option with care: a transient network error might cause the logging code
    /// to silently hang the whole program.
    ///
    #[serde(rename = "wait")]
    Wait,
    /// Discard new records if the async buffer is full.
    #[serde(rename = "discard")]
    Discard,
}

/// Struct to manipulate configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    level: GelfLevel,
    hostname: String,
    port: u64,
    null_character: bool,
    use_tls: bool,
    async_buffer_size: Option<usize>,
    buffer_size: Option<usize>,
    buffer_duration: Option<u64>,
    additional_fields: BTreeMap<Value, Value>,
    full_buffer_policy: Option<FullBufferPolicy>,
    connect_timeout_ms: Option<u64>,
    write_timeout_ms: Option<u64>,
}

impl Config {
    /// Load configuration using the given `path` file.
    /// ## Example
    ///
    /// ```rust
    /// use gelf_logger::Config;
    ///
    /// let config = Config::try_from_yaml("/tmp/myconf.yml").unwrap();
    /// ```
    pub fn try_from_yaml(path: &str) -> Result<Config> {
        Ok(serde_yaml::from_reader(File::open(path)?)?)
    }

    /// Shortcut to create a valid configuration for OVH [LDP](https://docs.ovh.com/gb/en/logs-data-platform/) service.
    ///
    /// ## Example
    ///
    /// ```toml
    /// [dependencies.gelf_logger]
    /// version = "*"
    /// features = ["ovh-ldp"]
    /// ```
    ///
    /// And then:
    ///
    /// ```rust
    /// use gelf_logger::Config;
    ///
    /// let cfg = Config::ldp("gra1.logs.ovh.com", "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX");
    ///```
    #[cfg(feature = "ovh-ldp")]
    pub fn ldp(cluster: &str, token: &str) -> Config {
        Config::builder()
            .set_hostname(cluster.to_string())
            .put_additional_field("X-OVH-TOKEN".to_string(), Value::String(token.to_string()))
            .set_use_tls(true)
            .set_port(12202)
            .build()
    }

    /// The threshold for this logger to level. Logging messages which are less severe than level
    /// will be ignored.
    pub fn level(&self) -> &GelfLevel {
        &self.level
    }
    /// The name of the remote server.
    pub fn hostname(&self) -> &String {
        &self.hostname
    }
    /// The port of the remote host.
    pub fn port(&self) -> &u64 {
        &self.port
    }
    /// Adds a NUL byte (`\0`) after each entry.
    pub fn null_character(&self) -> &bool {
        &self.null_character
    }
    /// Activate transport security.
    pub fn use_tls(&self) -> &bool {
        &self.use_tls
    }
    /// Get the asynchronous buffer size. This buffer is placed between the log subsystem
    /// and the network sender. This represent the maximum number of message the system
    /// will buffer before blocking while waiting for message to be actually sent to the
    /// remote server.
    ///
    /// If None is configured, it defaults to 1000
    pub fn async_buffer_size(&self) -> Option<usize> {
        self.async_buffer_size
    }
    /// Get the upperbound limit on the number of records that can be placed in the buffer, once
    /// this size has been reached, the buffer will be sent to the remote server.
    pub fn buffer_size(&self) -> &Option<usize> {
        &self.buffer_size
    }
    /// Get the maximum lifetime (in milli seconds) of the buffer before send it to the remote
    /// server.
    pub fn buffer_duration(&self) -> &Option<u64> {
        &self.buffer_duration
    }
    /// Every additional data which will be append to each log entry.
    pub fn additional_fields(&self) -> &BTreeMap<Value, Value> {
        &self.additional_fields
    }
    /// Get the full buffer policy
    pub fn full_buffer_policy(&self) -> Option<FullBufferPolicy> {
        self.full_buffer_policy
    }
    /// Get the write timeout in milliseconds
    pub fn write_timeout_ms(&self) -> Option<u64> {
        self.write_timeout_ms
    }
    /// Get the connect timeout in milliseconds
    pub fn connect_timeout_ms(&self) -> Option<u64> {
        self.connect_timeout_ms
    }

    /// Returns a new builder.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}
