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
pub struct GelfTcpOutput {
    hostname: String,
    port: u64,
    formatter: GelfFormatter,
    use_tls: bool,
    stream: Option<Box<dyn Write>>,
}

impl GelfTcpOutput {
    /// Create the TCP output
    pub fn new(
        hostname: String,
        port: u64,
        formatter: GelfFormatter,
        use_tls: bool,
    ) -> GelfTcpOutput {
        GelfTcpOutput {
            hostname,
            port,
            formatter,
            use_tls,
            stream: None,
        }
    }
    /// Write `GelfRecord` into TCP socket
    pub fn send(&mut self, data: &Vec<GelfRecord>) -> Result<()> {
        for rec in data.iter() {
            if let Ok(jdata) = self.formatter.format(rec) {
                self.write_stream(&jdata.as_bytes())?;
            }
        }
        Ok(())
    }

    fn write_stream(&mut self, bytes: &[u8]) -> Result<()> {
        if self.stream.is_none() {
            let address = format!("{}:{}", &self.hostname, &self.port);
            self.stream = Some(match self.use_tls {
                false => Box::new(TcpStream::connect(address)?),
                true => {
                    let connector = TlsConnector::new().unwrap();
                    let stream = TcpStream::connect(address)?;
                    Box::new(connector.connect(&self.hostname, stream)?)
                }
            })
        }
        if let Err(e) = self.stream.as_mut().unwrap().write(bytes) {
            // an error occured on the stream, reconnect it next time
            self.stream = None;
            Err(e)?;
        }
        Ok(())
    }
}

impl From<&Config> for GelfTcpOutput {
    fn from(cfg: &Config) -> GelfTcpOutput {
        GelfTcpOutput::new(
            cfg.hostname().clone(),
            cfg.port().clone(),
            GelfFormatter::from(cfg),
            cfg.use_tls().clone(),
        )
    }
}
