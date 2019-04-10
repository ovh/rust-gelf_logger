use std::collections::btree_map::BTreeMap;
use std::fs::File;

use serde_gelf::level::GelfLevel;

use crate::result::Result;

pub trait ConfigBuilder {
    fn new() -> Self;
    fn set_hostname(self, hostname: String) -> Self;
    fn set_port(self, port: u64) -> Self;
    fn set_null_character(self, null_character: bool) -> Self;
    fn set_use_tls(self, use_tls: bool) -> Self;
    fn set_buffer_size(self, buffer_size: usize) -> Self;
    fn set_buffer_duration(self, buffer_duration: u64) -> Self;
    fn put_additional_field(self, key: String, value: serde_value::Value) -> Self;
    fn extend_additional_fields(self, additional_fields: BTreeMap<String, serde_value::Value>) -> Self;
}

pub trait ConfigGetters {
    fn level(&self) -> &GelfLevel;
    fn hostname(&self) -> &String;
    fn port(&self) -> &u64;
    fn null_character(&self) -> &bool;
    fn use_tls(&self) -> &bool;
    fn buffer_size(&self) -> &Option<usize>;
    fn buffer_duration(&self) -> &Option<u64>;
    fn additional_fields(&self) -> &BTreeMap<String, serde_value::Value>;
}


#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    level: GelfLevel,
    hostname: String,
    port: u64,
    null_character: bool,
    use_tls: bool,
    buffer_size: Option<usize>,
    buffer_duration: Option<u64>,
    additional_fields: BTreeMap<String, serde_value::Value>,
}


impl ConfigBuilder for Config {
    fn new() -> Config {
        Config {
            level: GelfLevel::default(),
            hostname: "127.0.0.1".to_string(),
            port: 12202,
            null_character: true,
            use_tls: true,
            buffer_size: None,
            buffer_duration: None,
            additional_fields: BTreeMap::new(),
        }
    }
    fn set_hostname(mut self, hostname: String) -> Config {
        self.hostname = hostname;
        self
    }
    fn set_port(mut self, port: u64) -> Config {
        self.port = port;
        self
    }
    fn set_null_character(mut self, null_character: bool) -> Config {
        self.null_character = null_character;
        self
    }
    fn set_use_tls(mut self, use_tls: bool) -> Config {
        self.use_tls = use_tls;
        self
    }
    fn set_buffer_size(mut self, buffer_size: usize) -> Config {
        self.buffer_size = Some(buffer_size);
        self
    }
    fn set_buffer_duration(mut self, buffer_duration: u64) -> Config {
        self.buffer_duration = Some(buffer_duration);
        self
    }
    fn put_additional_field(mut self, key: String, value: serde_value::Value) -> Config {
        self.additional_fields.insert(key, value);
        self
    }
    fn extend_additional_fields(mut self, additional_fields: BTreeMap<String, serde_value::Value>) -> Config {
        self.additional_fields.extend(additional_fields);
        self
    }
}

impl ConfigGetters for Config {
    fn level(&self) -> &GelfLevel { &self.level }
    fn hostname(&self) -> &String { &self.hostname }
    fn port(&self) -> &u64 { &self.port }
    fn null_character(&self) -> &bool { &self.null_character }
    fn use_tls(&self) -> &bool { &self.use_tls }
    fn buffer_size(&self) -> &Option<usize> { &self.buffer_size }
    fn buffer_duration(&self) -> &Option<u64> { &self.buffer_duration }
    fn additional_fields(&self) -> &BTreeMap<String, serde_value::Value> { &self.additional_fields }
}

impl Config {
    pub fn try_from_yaml(path: &str) -> Result<Config> {
        Ok(serde_yaml::from_reader(File::open(path)?)?)
    }
    #[cfg(feature = "ovh-ldp")]
    pub fn ldp(cluster: &str, token: &str) -> Config {
        Config::new()
            .set_hostname(cluster.to_string())
            .put_additional_field("X-OVH-TOKEN".to_string(), serde_value::Value::String(token.to_string()))
            .set_use_tls(true)
            .set_port(12202)
    }
}
