use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;
use std::time::Duration;

use serde_gelf::record::GelfRecord;

use crate::output::GelfTcpOutput;
use crate::result::Error;

/***************************************************************************************************
// Event
***************************************************************************************************/


#[derive(Clone, Debug)]
pub enum Event {
    Send,
    Data(GelfRecord),
}


/***************************************************************************************************
// Metronome
***************************************************************************************************/


#[derive(Clone, Debug)]
pub struct Metronome {
    frequency: u64,
}


impl Metronome {
    pub fn new(frequency: u64) -> Metronome {
        Metronome { frequency }
    }
    pub fn start(&self, chan: SyncSender<Event>) {
        let frequency = self.frequency;
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(frequency));
            let _ = chan.send(Event::Send);
        });
    }
}

#[derive(Debug)]
pub struct Buffer {
    items: Vec<GelfRecord>,
    arx: Arc<Mutex<Receiver<Event>>>,
    errors: Vec<Error>,
    output: GelfTcpOutput,
}

impl Buffer {
    pub fn new(arx: Arc<Mutex<Receiver<Event>>>, output: GelfTcpOutput) -> Buffer {
        Buffer { items: Vec::new(), arx, errors: Vec::new(), output }
    }
    pub fn run(&mut self) {
        loop {
            match { self.arx.lock().unwrap().recv() } {
                Ok(event) => {
                    match event {
                        Event::Send => match self.output.send(&self.items) {
                            Ok(_) => self.items.clear(),
                            Err(exc) => {
                                self.errors.push(exc);
                                if self.errors.len() >= 5 {
                                    println!("Too many errors !");
                                    for err in self.errors.iter() {
                                        println!("{:?}", err);
                                    }
                                    std::process::exit(0x0100);
                                }
                                thread::sleep(Duration::from_millis(100));
                            }
                        },
                        Event::Data(record) => self.items.push(record),
                    }
                }
                Err(_) => return,
            };
        }
    }
}
