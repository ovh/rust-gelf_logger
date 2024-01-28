// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

use std::{
    fmt,
    sync::mpsc::{SendError, TrySendError},
};

use crate::buffer::Event;

/// Enum to represent errors
#[derive(Debug)]
pub enum Error {
    /// Error raised when  the channel gets disconnect or the async buffer is
    /// full
    FullChannelError(Event),
    /// Error raised if the program failed to send a record into the channel.
    ChannelDisconnectedError(Event),
    /// Error raised if the output failed to write in the TCP socket.
    IOError(std::io::Error),
    /// Error while JSON serialization.
    JsonSerializerError(serde_json::Error),
    /// Error raised is the program failed to initialize the logger.
    LogError(log::SetLoggerError),
    /// Error on TLS initialization.
    TLSError(native_tls::HandshakeError<std::net::TcpStream>),
    /// Invalid value
    ValueSerializerError(serde_value::SerializerError),
    /// Invalid yaml content
    #[cfg(feature = "yaml")]
    YamlError(serde_yaml::Error),
}

pub(crate) type Result<S> = std::result::Result<S, Error>;

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

#[cfg(feature = "yaml")]
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
        Error::ChannelDisconnectedError(err.0)
    }
}
impl From<TrySendError<Event>> for Error {
    fn from(err: TrySendError<Event>) -> Error {
        match err {
            TrySendError::Full(e) => Error::FullChannelError(e),
            TrySendError::Disconnected(e) => Error::ChannelDisconnectedError(e),
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IOError(err) => err.fmt(f),
            Error::JsonSerializerError(err) => err.fmt(f),
            Error::LogError(err) => err.fmt(f),
            Error::TLSError(err) => err.fmt(f),
            Error::ValueSerializerError(err) => err.fmt(f),
            #[cfg(feature = "yaml")]
            Error::YamlError(err) => err.fmt(f),
            Error::FullChannelError(e) => {
                write!(f, "Async channel buffer is full while sending {:?}", e)
            }
            Error::ChannelDisconnectedError(e) => write!(
                f,
                "Async channel buffer is disconnected while sending {:?}",
                e
            ),
        }
    }
}
