// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

//! # gelf_logger
//!
//! The Graylog Extended Log Format ([GELF](http://docs.graylog.org/en/latest/pages/gelf.html)) is a log format that avoids the shortcomings of classic
//! log formats. GELF is a great choice for logging from within applications.
//! There are libraries and appenders for many programming languages and logging
//! frameworks so it is easy to implement. You could use GELF to send every
//! exception as a log message to your Graylog cluster.
//!
//! The logger will:
//! 1. serialize log entries  using the [serde_gelf](https://crates.io/crates/serde_gelf)
//!    crate.
//! 2. bufferize the result into memory.
//! 3. batch send over network using TCP/TLS.
//!
//! ## Example
//!
//! ```rust
//! #[macro_use]
//! extern crate gelf_logger;
//! #[macro_use]
//! extern crate log;
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate serde_value;
//!
//! use serde_gelf::GelfLevel;
//!
//! use gelf_logger::Config;
//!
//! #[derive(Serialize)]
//! struct Myapp {
//!     name: String,
//!     version: String,
//! }
//!
//! impl Default for Myapp {
//!     fn default() -> Myapp {
//!         Myapp {
//!             name: env!("CARGO_PKG_NAME").into(),
//!             version: env!("CARGO_PKG_VERSION").into(),
//!         }
//!     }
//! }
//!
//! fn main() {
//!     let cfg = Config::builder()
//!         .set_hostname("localhost".into())
//!         .set_port(12202)
//!         .set_level(GelfLevel::Informational)
//!         .set_buffer_duration(300)
//!         .set_buffer_size(500)
//!         .put_additional_field("myValue".into(), serde_value::Value::I64(10))
//!         .set_null_character(true)
//!         .build();
//!
//!     // Initialize logger
//!     gelf_logger::init(cfg).unwrap();
//!
//!     // Send log using a macro defined in the create log
//!     info!("common message");
//!
//!     // Use a macro from gelf_logger to send additional data
//!     gelf_warn!(extra: &Myapp::default(), "My app info");
//!
//!     // make sure all buffered records are sent before exiting
//!     gelf_logger::flush().unwrap();
//! }
//! ```
#![doc(
    html_logo_url = "https://eu.api.ovh.com/images/com-square-bichro.png",
    html_favicon_url = "https://www.ovh.com/favicon.ico"
)]
#![deny(warnings, missing_docs)]

pub use batch::{flush, init, init_from_file, init_processor, processor, Batch, BatchProcessor};
pub use buffer::Buffer;
pub use config::{Config, ConfigBuilder};
pub use output::GelfTcpOutput;
pub use result::Error;

mod batch;
mod buffer;
mod config;
mod formatter;
mod logger;
mod macros;
mod output;
mod result;
