// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2009 The gelf_logger Authors. All rights reserved.

use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};

use native_tls::TlsConnector;
use serde_gelf::GelfRecord;

use crate::config::Config;
use crate::formatter::GelfFormatter;
use crate::result::Result;
use std::io;
use std::time::Duration;

/// Struct to send `GelfRecord` into a TCP socket
pub struct GelfTcpOutput {
    hostname: String,
    port: u64,
    formatter: GelfFormatter,
    use_tls: bool,
    stream: Option<Box<dyn Write>>,
    connect_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl GelfTcpOutput {
    /// Create the TCP output
    pub fn new(
        hostname: String,
        port: u64,
        formatter: GelfFormatter,
        use_tls: bool,
        connect_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    ) -> GelfTcpOutput {
        GelfTcpOutput {
            hostname,
            port,
            formatter,
            use_tls,
            stream: None,
            connect_timeout,
            write_timeout,
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
            self.stream = Some(match self.use_tls {
                false => Box::new(self.tcp_connect()?),
                true => {
                    let connector = TlsConnector::new().unwrap();
                    let stream = self.tcp_connect()?;
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

    fn tcp_connect(&self) -> io::Result<TcpStream> {
        let address = format!("{}:{}", &self.hostname, &self.port);
        let stream = match &self.connect_timeout {
            None => TcpStream::connect(address)?,
            Some(dur) => TcpStream::connect_timeout(
                &address.to_socket_addrs()?.next().unwrap(),
                dur.clone(),
            )?,
        };
        stream.set_write_timeout(self.write_timeout)?;
        Ok(stream)
    }
}

impl From<&Config> for GelfTcpOutput {
    fn from(cfg: &Config) -> GelfTcpOutput {
        GelfTcpOutput::new(
            cfg.hostname().clone(),
            cfg.port().clone(),
            GelfFormatter::from(cfg),
            cfg.use_tls().clone(),
            cfg.connect_timeout_ms().map(|ms| Duration::from_millis(ms)),
            cfg.write_timeout_ms().map(|ms| Duration::from_millis(ms)),
        )
    }
}
