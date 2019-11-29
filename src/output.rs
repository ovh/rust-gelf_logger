// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2009 The gelf_logger Authors. All rights reserved.

use std::io::Write;
use std::net::TcpStream;

use native_tls::TlsConnector;
use serde_gelf::GelfRecord;

use crate::config::Config;
use crate::formatter::GelfFormatter;
use crate::result::Result;

/// Struct to send `GelfRecord` into a TCP socket
#[derive(Clone, Debug)]
pub struct GelfTcpOutput {
    hostname: String,
    port: u64,
    formatter: GelfFormatter,
    use_tls: bool,
}

impl GelfTcpOutput {
    /// Create the TCP output
    pub fn new(hostname: String, port: u64, formatter: GelfFormatter, use_tls: bool) -> GelfTcpOutput {
        GelfTcpOutput { hostname, port, formatter, use_tls }
    }
    /// Write `GelfRecord` into TCP socket
    pub fn send(&self, data: &Vec<GelfRecord>) -> Result<()> {
        let address = format!("{}:{}", &self.hostname, &self.port);
        match self.use_tls {
            false => {
                let mut stream = TcpStream::connect(address)?;
                for rec in data.iter() {
                    if let Ok(jdata) = self.formatter.format(rec) {
                        stream.write(jdata.as_bytes())?;
                    }
                }
            }
            true => {
                let connector = TlsConnector::new().unwrap();
                let stream = TcpStream::connect(address)?;
                let mut stream = connector.connect(&self.hostname, stream)?;

                for rec in data.iter() {
                    if let Ok(jdata) = self.formatter.format(rec) {
                        stream.write(jdata.as_bytes())?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl From<&Config> for GelfTcpOutput {
    fn from(cfg: &Config) -> GelfTcpOutput {
        GelfTcpOutput::new(cfg.hostname().clone(), cfg.port().clone(), GelfFormatter::from(cfg), cfg.use_tls().clone())
    }
}