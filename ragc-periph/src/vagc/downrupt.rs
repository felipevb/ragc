use super::generate_yaagc_packet;

use crossbeam_channel::{unbounded, Receiver, Sender};
use std::io::Write;
use std::net::TcpListener;

use ragc_core::mem::periph::AgcIoPeriph;

pub struct DownruptPeriph {
    tx: Sender<[u8; 4]>,
    word_order: bool
}

fn downrupt_thread(rx: Receiver<[u8; 4]>, addr: &str) {
    // accept connections and process them serially
    let listener = TcpListener::bind(addr).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut xa) => loop {
                let msg = match rx.recv() {
                    Ok(x) => x,
                    _ => {
                        break;
                    }
                };

                match xa.write_all(&msg) {
                    Ok(_x) => {}
                    _ => {
                        break;
                    }
                }
            },
            _ => {}
        };
    }
}

///
/// ## DownruptPeriph Module
///
/// The DownruptPeriph module is used to keep track of the downrupt message
/// and send it back to ground control once all of it is collected. In reality,
/// each double word is sent every 20ms. For this, we will buffer the entire
/// message before we send it out. Also, keeps an eye for signatures within
/// the start of the message.
///
impl DownruptPeriph {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();

        std::thread::spawn(move || downrupt_thread(rx, "127.0.0.1:19800"));
        DownruptPeriph {
            tx: tx,
            word_order: false
        }
    }
}

impl AgcIoPeriph for DownruptPeriph {
    ///
    /// Implementing the `read` function for the Peripherial. For the Downrupt
    /// peripherial, there is no `read`s that occur. For this, we just return
    /// 0 to the code.
    ///
    fn read(&self, channel_idx: usize) -> u16 {
        match channel_idx {
            ragc_core::consts::io::CHANNEL_CHAN13 => {
                if self.word_order {
                    1 << 6
                }
                else {
                    0o00000
                }
            }
            ragc_core::consts::io::CHANNEL_CHAN30 |
            ragc_core::consts::io::CHANNEL_CHAN31 |
            ragc_core::consts::io::CHANNEL_CHAN32 |
            ragc_core::consts::io::CHANNEL_CHAN33 |
            ragc_core::consts::io::CHANNEL_CHAN34 |
            ragc_core::consts::io::CHANNEL_CHAN35 => { 0o77777 }
            _ => { 0o00000 }
        }
    }

    ///
    /// Implementing the `write` function for the Peripherial. This write does
    /// not distinguish between DOWNRUPT WORD1 and DOWNRUPT WORD2 addresses.
    /// Whichever order the code writes to the downrupt is reflected in the state
    /// vector.
    ///
    /// Assumption: Code does not write to DOWNRUPT2 before DOWNRUPT1 word
    ///
    fn write(&mut self, channel_idx: usize, value: u16) {
        match channel_idx {
            ragc_core::consts::io::CHANNEL_CHAN13 => {
                if value & (1 << 6) != 0o00000 {
                    self.word_order = true;
                }
                else {
                    self.word_order = false;
                }
            },
            ragc_core::consts::io::CHANNEL_CHAN34 => {
                let packet = generate_yaagc_packet(channel_idx, value);
                self.tx.send(packet).unwrap();
            }
            ragc_core::consts::io::CHANNEL_CHAN35 => {
                let packet = generate_yaagc_packet(channel_idx, value);
                self.tx.send(packet).unwrap();
            }
            _ => {}
        }
    }

    fn is_interrupt(&mut self) -> u16 {
        0
    }
}
