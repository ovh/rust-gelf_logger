use crate::buffer::Event;
use std::sync::mpsc::SendError;

#[derive(Debug)]
pub enum Error {
    FileNotFound(String),
    InvalidOutput(String),
    IOError(std::io::Error),
    JsonSerializerError(serde_json::Error),
    LogError(log::SetLoggerError),
    NotInitialized(String),
    ValueSerializerError(serde_value::SerializerError),
    YamlError(serde_yaml::Error),
    ChannelError(SendError<Event>),
}

pub type Result<S> = std::result::Result<S, Error>;

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