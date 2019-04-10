use std::collections::btree_map::BTreeMap;

use serde_gelf::record::GelfRecord;
use serde_gelf::record::GelfRecordBuilder;
use serde_gelf::ser::GelfField;

use crate::config::{Config, ConfigGetters};
use crate::result::Result;

#[derive(Clone, Debug)]
pub struct GelfFormatter {
    additional_fields: BTreeMap<String, serde_value::Value>,
    null_character: bool,
}

impl GelfFormatter {
    pub fn new(null_character: bool, additional_fields: BTreeMap<String, serde_value::Value>) -> GelfFormatter {
        GelfFormatter {
            null_character,
            additional_fields: match serde_value::to_value(&additional_fields) {
                Err(_) => BTreeMap::new(),
                Ok(values) => GelfField::new("", "", &values).disassemble()
            },
        }
    }
    fn default_additional_fields(&self) -> &BTreeMap<String, serde_value::Value> {
        &self.additional_fields
    }

    fn null_character(&self) -> &bool {
        &self.null_character
    }

    pub fn format(&self, record: &GelfRecord) -> Result<String> {
        let rec = record.clone()
            .extend_additional_fields(self.default_additional_fields().clone());
        Ok(match self.null_character() {
            &true => format!("{}\n\0", serde_json::to_string(&rec)?),
            &false => format!("{}\n", serde_json::to_string(&rec)?)
        })
    }
}

impl From<&Config> for GelfFormatter {
    fn from(cfg: &Config) -> GelfFormatter {
        GelfFormatter::new(cfg.null_character().clone(), cfg.additional_fields().clone())
    }
}