// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

use std::collections::BTreeMap;

use serde_gelf::{GelfRecord, GelfRecordBuilder};
use serde_value::Value;
use serde_value_utils::to_flatten_maptree;

use crate::{config::Config, result::Result};

#[derive(Clone, Debug)]
pub struct GelfFormatter {
    additional_fields: BTreeMap<Value, Value>,
    null_character: bool,
}

impl GelfFormatter {
    pub fn new(null_character: bool, additional_fields: BTreeMap<Value, Value>) -> GelfFormatter {
        GelfFormatter {
            null_character,
            additional_fields: to_flatten_maptree("_", Some("_"), &additional_fields)
                .unwrap_or_else(|_| BTreeMap::new()),
        }
    }
    fn default_additional_fields(&self) -> &BTreeMap<Value, Value> {
        &self.additional_fields
    }

    fn null_character(&self) -> &bool {
        &self.null_character
    }

    pub fn format(&self, record: &GelfRecord) -> Result<String> {
        let rec = record
            .clone()
            .extend_additional_fields(self.default_additional_fields().clone());
        Ok(match self.null_character() {
            &true => format!("{}\n\0", serde_json::to_string(&rec)?),
            &false => format!("{}\n", serde_json::to_string(&rec)?),
        })
    }
}

impl From<&Config> for GelfFormatter {
    fn from(cfg: &Config) -> GelfFormatter {
        GelfFormatter::new(
            cfg.null_character().clone(),
            cfg.additional_fields().clone(),
        )
    }
}
