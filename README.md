# gelf_logger 

[![Build Status](https://travis-ci.org/ovh/rust-gelf_logger.svg?branch=master)](https://travis-ci.org/ovh/rust-gelf_logger) 
[![Latest version](https://img.shields.io/crates/v/gelf_logger.svg)](https://crates.io/crates/gelf_logger) 
[![Documentation](https://docs.rs/gelf_logger/badge.svg)](https://docs.rs/gelf_logger) 
![License](https://img.shields.io/crates/l/gelf_logger.svg)

The Graylog Extended Log Format ([GELF](http://docs.graylog.org/en/latest/pages/gelf.html)) is a log format that avoids the shortcomings of classic
log formats. GELF is a great choice for logging from within applications. There are libraries
and appenders for many programming languages and logging frameworks so it is easy to implement.
You could use GELF to send every exception as a log message to your Graylog cluster.

The logger will:
1. serialize log entries  using the [serde_gelf](https://crates.io/crates/serde_gelf) crate.
2. bufferize the result into memory.
3. batch send over network using TCP/TLS.

## Example

```rust
use std::time::Duration;

use gelf_logger::{gelf_warn, Config, GelfLevel};
use log::info;
use serde_derive::Serialize;

#[derive(Serialize)]
struct Myapp {
    name: String,
    version: String,
}

impl Default for Myapp {
    fn default() -> Myapp {
        Myapp {
            name: env!("CARGO_PKG_NAME").into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

fn main() {
    let cfg = Config::builder()
        .set_hostname("localhost".into())
        .set_port(12202)
        .set_level(GelfLevel::Informational)
        .set_buffer_duration(Duration::from_millis(300))
        .set_buffer_size(500)
        .put_additional_field("myValue".into(), gelf_logger::Value::I64(10))
        .set_null_character(true)
        .build();

    // Initialize logger
    gelf_logger::init(cfg).unwrap();

    // Send log using a macro defined in the create log
    info!("common message");

    // Use a macro from gelf_logger to send additional data
    gelf_warn!(extra: &Myapp::default(), "My app info");

    // make sure all buffered records are sent before exiting
    gelf_logger::flush().unwrap();
}
```

## License

Licensed under [BSD 3-Clause License](./LICENSE) or (https://opensource.org/licenses/BSD-3-Clause)
