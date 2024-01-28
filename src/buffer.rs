// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
// Copyright 2024 The gelf_logger Authors. All rights reserved.

use std::{
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::{Duration, Instant},
};

use serde_gelf::GelfRecord;

use crate::{output::GelfTcpOutput, result::Error};

/// Enum used to send commands over the channel.
#[derive(Debug)]
pub enum Event {
    /// Command to force the flush of the buffer.
    Flush,
    /// Command used to send a record into the buffer.
    Data(GelfRecord),
}

/// struct to store a buffer of `GelfRecord`
pub struct Buffer {
    items: Vec<GelfRecord>,
    rx: Receiver<Event>,
    errors: Vec<Error>,
    output: GelfTcpOutput,
    buffer_size: Option<usize>,
    buffer_duration: Option<Duration>,
}

impl Buffer {
    /// Initialize buffer
    pub fn new(
        rx: Receiver<Event>,
        output: GelfTcpOutput,
        buffer_size: Option<usize>,
        buffer_duration: Option<Duration>,
    ) -> Buffer {
        Buffer {
            items: Vec::new(),
            errors: Vec::new(),
            rx,
            output,
            buffer_size,
            buffer_duration,
        }
    }
    /// Buffer body (loop)
    pub fn run(&mut self) {
        let mut last_send = Instant::now();
        loop {
            let time_to_wait = self
                .buffer_duration
                .map(|duration| duration.checked_sub(Instant::now().duration_since(last_send)))
                .unwrap_or(None); // flatten() would have been more explicit here bit it is not stable yet

            let event = match time_to_wait {
                None => match self.rx.recv() {
                    Ok(e) => e,
                    Err(_) => return,
                },
                Some(duration) => match self.rx.recv_timeout(duration) {
                    Ok(e) => e,
                    Err(e) => match e {
                        RecvTimeoutError::Timeout => Event::Flush,
                        RecvTimeoutError::Disconnected => return,
                    },
                },
            };

            match event {
                Event::Flush => self.flush(),
                Event::Data(record) => {
                    self.items.push(record);
                    if let Some(max_buffer_size) = self.buffer_size {
                        if self.items.len() >= max_buffer_size {
                            self.flush();
                        }
                    }
                }
            }
            last_send = Instant::now();
        }
    }

    fn flush(&mut self) {
        match self.output.send(&self.items) {
            Ok(_) => self.items.clear(),
            Err(exc) => {
                self.errors.push(exc);
                if self.errors.len() >= 5 {
                    eprintln!("Many errors occurred while sending GELF logs event!");
                    for err in self.errors.iter() {
                        eprintln!(">> {:?}", err);
                    }
                    self.errors.clear();
                }
            }
        }
    }
}
