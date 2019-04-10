use crate::buffer::Event;
use std::sync::mpsc::SendError;

#[derive(Debug)]
pub enum Error {
    ChannelError(SendError<Event>),
    FileNotFound(String),
    InvalidOutput(String),
    IOError(std::io::Error),
    JsonSerializerError(serde_json::Error),
    LogError(log::SetLoggerError),
    NotInitialized(String),
    TLSError(native_tls::HandshakeError<std::net::TcpStream>),
    ValueSerializerError(serde_value::SerializerError),
    YamlError(serde_yaml::Error),
}

pub type Result<S> = std::result::Result<S, Error>;

impl From<native_tls::HandshakeError<std::net::TcpStream>> for Error {
    fn from(err: native_tls::HandshakeError<std::net::TcpStream>) -> Error {
        Error::TLSError(err)
    }
}

impl From<log::SetLoggerError> for Error {
    fn from(err: log::SetLoggerError) -> Error {
        Error::LogError(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Error {
        Error::YamlError(err)
    }
}


impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<serde_value::SerializerError> for Error {
    fn from(err: serde_value::SerializerError) -> Error {
        Error::ValueSerializerError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonSerializerError(err)
    }
}

impl From<SendError<Event>> for Error {
    fn from(err: SendError<Event>) -> Error {
        Error::ChannelError(err)
    }
}