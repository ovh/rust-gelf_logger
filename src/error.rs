// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

use std::{io, net::TcpStream};

use log::SetLoggerError;
use thiserror::Error as ThisError;

/// Errors that can occur when using this crate.
#[allow(missing_docs)]
#[derive(ThisError, Debug)]
// TODO: Add full buffer error + strategy.
pub enum Error {
    #[error("logger already set")]
    AlreadySet(#[from] SetLoggerError),
    #[error("io failure")]
    Io(#[from] io::Error),
    #[error("tls handshake failure")]
    TlsHandshake(#[from] native_tls::HandshakeError<TcpStream>),
    #[error("tls connection failure")]
    Tls(#[from] native_tls::Error),
}
