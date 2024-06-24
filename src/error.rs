// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

use std::{io, net::TcpStream};

use log::SetLoggerError;
use thiserror::Error as ThisError;

/// Errors that can occur when using this crate.
#[derive(ThisError, Debug)]
// TODO: Add full buffer error + strategy.
pub enum Error {
    /// Occurs when trying to set the logger while another one is already set.
    #[error("logger already set")]
    AlreadySet(#[from] SetLoggerError),
    /// Occurs when any open, write or flush calls fail.
    #[error("io failure")]
    Io(#[from] io::Error),
    /// Occurs when the TLS handshake fails.
    #[error("tls handshake failure")]
    TlsHandshake(#[from] native_tls::HandshakeError<TcpStream>),
    /// Occurs when any TLS error happen.
    #[error("tls connection failure")]
    Tls(#[from] native_tls::Error),
}
