// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

use std::{
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

use log::{
    kv::{Error as KvError, Key, VisitSource},
    Record,
};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::level::GelfLevel;

#[doc(hidden)]
pub static INTERNAL_LEVEL_FIELD_NAME: &str = "__private_level";
const GELF_VERSION: &str = "1.1";

#[allow(missing_docs)]
#[derive(Serialize, Clone, Debug)]
pub struct GelfRecord<'a> {
    pub version: &'static str,
    pub host: &'static str,
    pub short_message: String,
    pub timestamp: Option<f64>,
    pub level: Option<u32>,
    #[serde(rename = "_levelname")]
    pub level_name: Option<&'static str>,
    #[serde(rename = "_facility")]
    pub facility: Option<&'a str>,
    #[serde(rename = "_line")]
    pub line: Option<u32>,
    #[serde(rename = "_file")]
    pub file: Option<&'a str>,
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

impl<'a> GelfRecord<'a> {
    /// Flatten, format and add fields to the record.
    pub fn extend_additional_fields(&mut self, fields: Map<String, Value>, type_suffix: bool) {
        self.additional_fields
            .extend(flatten(fields, Some("_"), "_", type_suffix));
    }
}

/// Convert a [`Record`] into a [`GelfRecord`]. The level specified in the
/// `Record` will be used to derive the `GelfRecord` one. If the special `kv`
/// value inserted by the `gelf_*` macros is present and is an integer, this
/// value will be used as `GelfLevel` instead.
impl<'a> From<&Record<'a>> for GelfRecord<'a> {
    fn from(record: &Record<'a>) -> Self {
        struct Visitor(Map<String, Value>, Option<GelfLevel>);

        impl<'a> VisitSource<'a> for Visitor {
            fn visit_pair(
                &mut self,
                key: Key<'a>,
                value: log::kv::Value<'a>,
            ) -> Result<(), KvError> {
                let json_value = serde_json::to_value(value).map_err(KvError::boxed)?;
                if key.as_str() == INTERNAL_LEVEL_FIELD_NAME && json_value.is_u64() {
                    self.1 = Some(GelfLevel::from(json_value.as_u64().unwrap() as u32));
                } else {
                    self.0.insert(key.as_str().to_owned(), json_value);
                }
                Ok(())
            }
        }

        let kvs = record.key_values();
        let mut visitor = Visitor(Map::with_capacity(kvs.count()), None);
        let _ = kvs.visit(&mut visitor);

        let level = GelfLevel::from(record.level());
        Self {
            version: GELF_VERSION,
            host: hostname(),
            short_message: record.args().to_string(),
            timestamp: Some(now()),
            level: Some(visitor.1.unwrap_or(level) as u32),
            level_name: Some(<&str>::from(visitor.1.unwrap_or(level))),
            facility: Some(record.target()),
            line: record.line(),
            file: record.file(),
            additional_fields: flatten(visitor.0, Some("_"), "_", true),
        }
    }
}

#[inline(always)]
fn hostname() -> &'static str {
    static CELL: OnceLock<String> = OnceLock::new();
    CELL.get_or_init(|| {
        hostname::get()
            .ok()
            .and_then(|h| h.to_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| "localhost".to_owned())
    })
    .as_str()
}

/// Default timestamp in seconds since UNIX epoch with optional decimal places
/// for milliseconds.
#[inline(always)]
fn now() -> f64 {
    // TODO: check if too much precision is fine or not.
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

/// `type_suffix`: https://help.ovhcloud.com/csm/en-logs-data-platform-field-naming-conventions?id=kb_article_view&sysparm_article=KB0055662
pub(crate) fn flatten(
    input: Map<String, Value>,
    prefix: Option<&str>,
    separator: &str,
    type_suffix: bool,
) -> Map<String, Value> {
    let mut path = Vec::with_capacity(8);
    if let Some(prefix) = prefix {
        path.push(prefix.to_owned());
    }

    fn process(
        buffer: &mut Map<String, Value>,
        path: &mut Vec<String>,
        current: Value,
        separator: &str,
        type_suffix: bool,
    ) {
        match current {
            Value::Array(array) => {
                path.push(separator.to_owned());
                for (i, v) in array.into_iter().enumerate() {
                    path.push(i.to_string());
                    process(buffer, path, v, separator, type_suffix);
                    path.pop();
                }
                path.pop();
            }
            Value::Object(sub_map) => {
                path.push(separator.to_owned());
                for (k, v) in sub_map {
                    path.push(k);
                    process(buffer, path, v, separator, type_suffix);
                    path.pop();
                }
                path.pop();
            }
            current => {
                let mut key = path.join("");
                if type_suffix {
                    key += match &current {
                        Value::Number(n) if n.is_f64() => "_float",
                        Value::Number(_) => "_long",
                        Value::Bool(_) => "_bool",
                        _ => "",
                    };
                }
                buffer.insert(key, current);
            }
        }
    }

    let mut buffer = Map::with_capacity(input.len());
    for (k, v) in input {
        path.push(k);
        process(&mut buffer, &mut path, v, separator, type_suffix);
        path.pop();
    }

    buffer
}

#[cfg(test)]
mod tests {
    use log::{kv::ToValue, Level, Record};
    use serde_json::{json, Map, Value};

    use super::{flatten, GelfRecord, GELF_VERSION};

    #[test]
    fn record() {
        // This is similar to what is done by the `log::error!` macro.
        let kvs = [("key_1", "value_1".to_value()), ("key_2", 3.to_value())];
        let record = Record::builder()
            .args(format_args!("something happen"))
            .level(Level::Error)
            .target(module_path!())
            .file_static(Some(file!()))
            .line(Some(line!()))
            .module_path_static(Some(module_path!()))
            .key_values(&kvs)
            .build();

        let gelf_record = GelfRecord::from(&record);
        assert_eq!(
            serde_json::to_value(&gelf_record).unwrap(),
            json!({
                "version": GELF_VERSION,
                "host": super::hostname(),
                "short_message": "something happen",
                "timestamp": gelf_record.timestamp,
                "level": 3,
                "_levelname": "Error",
                "_facility": module_path!(),
                "_line": record.line(),
                "_file": file!(),
                "_key_1": "value_1",
                "_key_2_long": 3,
            })
        );
    }

    fn json_to_map(value: Value) -> Map<String, Value> {
        match value {
            Value::Object(map) => map,
            _ => panic!("not a map"),
        }
    }

    #[test]
    fn already_flatten() {
        assert_eq!(
            flatten(
                json_to_map(json!({
                    "a": 1,
                    "b": "c"
                })),
                None,
                "_",
                false
            ),
            json_to_map(json!({
                "a": 1,
                "b": "c"
            }))
        );
    }

    #[test]
    fn already_flatten_add_prefix() {
        assert_eq!(
            flatten(
                json_to_map(json!({
                    "a": 1,
                    "b": "c"
                })),
                Some("_"),
                "_",
                false
            ),
            json_to_map(json!({
                "_a": 1,
                "_b": "c"
            }))
        );
    }

    #[test]
    fn depth_two() {
        assert_eq!(
            flatten(
                json_to_map(json!({
                    "a": 1,
                    "b": {
                        "c": "d",
                        "d": "f"
                    },
                    "e": 2
                })),
                None,
                "_",
                false
            ),
            json_to_map(json!({
                "a": 1,
                "b_c": "d",
                "b_d": "f",
                "e": 2
            }))
        );
        assert_eq!(
            flatten(
                json_to_map(json!({
                    "a": 1,
                    "b": {
                        "c": "d",
                        "d": "f"
                    },
                    "e": 2
                })),
                Some("_"),
                "_",
                false
            ),
            json_to_map(json!({
                "_a": 1,
                "_b_c": "d",
                "_b_d": "f",
                "_e": 2
            }))
        );
    }

    #[test]
    fn type_suffix() {
        assert_eq!(
            flatten(
                json_to_map(json!({
                    "a": 1,
                    "b": "c",
                    "c": true,
                    "d": 3.14
                })),
                None,
                "_",
                true
            ),
            json_to_map(json!({
                "a_long": 1,
                "b": "c",
                "c_bool": true,
                "d_float": 3.14
            }))
        );
        assert_eq!(
            flatten(
                json_to_map(json!({
                    "a": 1,
                    "b": {
                        "c": true,
                        "d": 3.14
                    },
                    "e": "f"
                })),
                None,
                "_",
                true
            ),
            json_to_map(json!({
                "a_long": 1,
                "b_c_bool": true,
                "b_d_float": 3.14,
                "e": "f"
            }))
        );
    }
}
