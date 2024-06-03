// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

use std::{
    io,
    io::Write,
    net::{TcpStream, ToSocketAddrs},
    sync::mpsc,
    thread,
    time::Duration,
};

use env_filter::Filter;
use log::{LevelFilter, Log, Metadata, Record};
use native_tls::{TlsConnector, TlsStream};
use serde_json::{Map, Value};

use crate::{error::Error, record::GelfRecord, Builder};

/// A logger what will format and forward any [`Record`] to the set-up target.
#[derive(Debug)]
pub struct GelfLogger {
    pub(crate) filter: Filter,
    pub(crate) writer: Writer,
    pub(crate) null_character: bool,
    pub(crate) additional_fields: Map<String, Value>,
}

impl GelfLogger {
    /// Crate a new Builder.
    ///
    /// This is equivalent to calling [`Builder::default()`].
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Returns the maximum `LevelFilter` that this env logger instance is
    /// configured to output.
    pub fn filter(&self) -> LevelFilter {
        self.filter.filter()
    }

    /// Checks if this record matches the configured filter.
    pub fn matches(&self, record: &Record<'_>) -> bool {
        self.filter.matches(record)
    }
}

impl Log for GelfLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.filter.enabled(metadata)
    }

    fn log(&self, record: &Record<'_>) {
        if !self.matches(record) {
            return;
        }

        let mut record = GelfRecord::from(record);
        record
            .additional_fields
            .extend(self.additional_fields.clone());

        let Ok(mut data) = serde_json::to_vec(&record) else {
            return;
        };

        data.push(b'\n');
        if self.null_character {
            data.push(b'\0');
        }

        self.writer.write(Op::Data(data));
    }

    fn flush(&self) {
        let (tx, rx) = mpsc::sync_channel(1);
        self.writer.write(Op::Flush(tx));
        let _ = rx.recv();
    }
}

impl Drop for GelfLogger {
    fn drop(&mut self) {
        self.flush();
    }
}

#[derive(Debug)]
pub(crate) enum Writer {
    Stdout,
    Stderr,
    Pipe(mpsc::SyncSender<Op>),
}

impl Writer {
    pub(crate) fn new(target: Target) -> Result<Self, Error> {
        Ok(match target {
            Target::Stdout => Self::Stdout,
            Target::Stderr => Self::Stderr,
            Target::Tcp(TcpTarget {
                hostname,
                port,
                tls,
                connect_timeout,
                write_timeout,
                buffer_size,
                background_error_handler,
            }) => {
                let (tx, rx) = mpsc::sync_channel::<Op>(buffer_size);
                thread::spawn(move || {
                    let mut conn = None;
                    while let Ok(op) = rx.recv() {
                        if conn.is_none() {
                            conn = handle_background_error(
                                background_error_handler,
                                TcpConnection::new(
                                    &hostname,
                                    port,
                                    tls,
                                    connect_timeout,
                                    write_timeout,
                                ),
                            );
                        }

                        if let Some(conn_ref) = &mut conn {
                            match op {
                                Op::Data(data) => {
                                    if handle_background_error(
                                        background_error_handler,
                                        conn_ref.write_all(&data),
                                    )
                                    .is_none()
                                    {
                                        conn = None;
                                    }
                                }
                                Op::Flush(tx) => {
                                    if handle_background_error(
                                        background_error_handler,
                                        conn_ref.flush(),
                                    )
                                    .is_none()
                                    {
                                        conn = None;
                                    }
                                    let _ = tx.send(());
                                }
                            }
                        }
                    }
                });
                Self::Pipe(tx)
            }
        })
    }

    fn write(&self, op: Op) {
        match op {
            Op::Data(data) => match self {
                Writer::Stdout => {
                    let _ = io::stdout().write_all(&data);
                }
                Writer::Stderr => {
                    let _ = io::stderr().write_all(&data);
                }
                Writer::Pipe(tx) => {
                    let _ = tx.send(Op::Data(data));
                }
            },
            Op::Flush(flush_tx) => match self {
                Writer::Stdout => {
                    let _ = io::stdout().flush();
                    let _ = flush_tx.send(());
                }
                Writer::Stderr => {
                    let _ = io::stderr().flush();
                    let _ = flush_tx.send(());
                }
                Writer::Pipe(tx) => {
                    let _ = tx.send(Op::Flush(flush_tx));
                }
            },
        }
    }
}

pub(crate) enum Op {
    Data(Vec<u8>),
    Flush(mpsc::SyncSender<()>),
}

/// The output target used by a [`GelfLogger`].
#[derive(Clone, Debug)]
pub enum Target {
    /// GELF records will be printed to stdout.
    Stdout,
    /// GELF records will be printed to stderr.
    Stderr,
    /// GELF records will be forwarded over TCP.
    Tcp(TcpTarget),
}

/// A TCP target used to send the GELF records.
#[derive(Clone, Debug)]
pub struct TcpTarget {
    /// The hostname used to resolve the remote host and establish the TLS
    /// handshake if requested.
    pub hostname: String,
    /// The remote port to connect to.
    pub port: u16,
    /// Whether to use TLS over TCP. The hostname specified above will be used
    /// to perform the TLS handshake.
    pub tls: bool,
    /// Set the connection timeout duration. If `None` is specified, the socket
    /// connection phase can block indefinitely.
    pub connect_timeout: Option<Duration>,
    /// Set the connection write timeout duration. If `None` is specified, the
    /// socket write calls can block indefinitely.
    pub write_timeout: Option<Duration>,
    /// Set the number of messages that can be queued between the caller and
    /// background threads. If too many log calls are made and the background is
    /// too slow, this buffer will fill up. When full, calls on the current
    /// thread will start to block.
    pub buffer_size: usize,
    /// Register a static function that will be called when errors occur in the
    /// background thread.
    pub background_error_handler: Option<fn(Error)>,
}

impl Default for TcpTarget {
    /// Crate TCP target with the following placeholders:
    /// ```rust,ignore
    /// TcpTarget {
    ///     hostname: "127.0.0.1".to_owned(),
    ///     port: 2202,
    ///     tls: false,
    ///     connect_timeout: None,
    ///     write_timeout: None,
    ///     buffer_size: 1_000,
    ///     background_error_handler: None,
    /// }
    /// ```
    fn default() -> Self {
        Self {
            hostname: "127.0.0.1".to_owned(),
            port: 2202,
            tls: false,
            connect_timeout: None,
            write_timeout: None,
            buffer_size: 1_000,
            background_error_handler: None,
        }
    }
}

enum TcpConnection {
    Raw(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl TcpConnection {
    fn new(
        hostname: &str,
        port: u16,
        tls: bool,
        connect_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    ) -> Result<Self, Error> {
        let socket_addr = (hostname, port).to_socket_addrs().unwrap().next().unwrap();
        let stream = match connect_timeout {
            Some(timeout) => TcpStream::connect_timeout(&socket_addr, timeout),
            None => TcpStream::connect(socket_addr),
        }?;
        stream.set_write_timeout(write_timeout)?;

        Ok(if tls {
            let connector = TlsConnector::new()?;
            Self::Tls(connector.connect(hostname, stream)?)
        } else {
            Self::Raw(stream)
        })
    }

    fn write_all(&mut self, data: &[u8]) -> Result<(), io::Error> {
        match self {
            TcpConnection::Raw(stream) => stream.write_all(data),
            TcpConnection::Tls(stream) => stream.write_all(data),
        }
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        match self {
            TcpConnection::Raw(stream) => stream.flush(),
            TcpConnection::Tls(stream) => stream.flush(),
        }
    }
}

fn handle_background_error<T, E: Into<Error>>(
    handler: Option<fn(Error)>,
    error: Result<T, E>,
) -> Option<T> {
    match (handler, error) {
        (Some(handler), Err(err)) => {
            handler(err.into());
            None
        }
        (_, Ok(value)) => Some(value),
        _ => None,
    }
}
