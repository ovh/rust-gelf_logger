use std::io::Write;
use std::net::TcpStream;

use serde_gelf::record::GelfRecord;

use crate::config::{Config, ConfigGetters};
use crate::formatter::GelfFormatter;

#[derive(Clone, Debug)]
pub struct GelfTcpOutput {
    hostname: String,
    port: u64,
    formatter: GelfFormatter,
}

impl GelfTcpOutput {
    pub fn new(hostname: String, port: u64, formatter: GelfFormatter) -> GelfTcpOutput {
        GelfTcpOutput { hostname, port, formatter }
    }
    pub fn send(&self, data: &Vec<GelfRecord>) -> std::io::Result<()> {
        let address = format!("{}:{}", self.hostname, self.port);
        let mut socket = TcpStream::connect(address.as_str())?;
        for rec in data.iter() {
            if let Ok(jdata) = self.formatter.format(rec) {
                socket.write(jdata.as_bytes())?;
            }
        }
        Ok(())
    }
}

impl From<&Config> for GelfTcpOutput {
    fn from(cfg: &Config) -> GelfTcpOutput {
        GelfTcpOutput::new(cfg.hostname().clone(), cfg.port().clone(), GelfFormatter::from(cfg))
    }
}