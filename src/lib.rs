#![deny(warnings)]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_gelf;
extern crate serde_json;
extern crate serde_value;
extern crate serde_yaml;


mod batch;
mod buffer;
pub mod config;
mod formatter;
mod logger;
mod macros;
mod output;
mod result;


pub use batch::{processor, init, init_from_file};