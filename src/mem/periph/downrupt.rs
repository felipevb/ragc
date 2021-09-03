use crate::utils::generate_yaagc_packet;

use crossbeam_channel::{unbounded, Receiver, Sender};
use std::io::Write;
use std::net::TcpListener;

pub struct DownruptPeriph {
    tx: Sender<[u8; 4]>,
}

fn downrupt_thread(rx: Receiver<[u8; 4]>) {
    // accept connections and process them serially
    let listener = TcpListener::bind("127.0.0.1:19800").unwrap();
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

        std::thread::spawn(move || downrupt_thread(rx));
        DownruptPeriph { tx: tx }
    }

    ///
    /// Implementing the `read` function for the Peripherial. For the Downrupt
    /// peripherial, there is no `read`s that occur. For this, we just return
    /// 0 to the code.
    ///
    pub fn read(&self, _channel_idx: usize) -> u16 {
        0
    }

    ///
    /// Implementing the `write` function for the Peripherial. This write does
    /// not distinguish between DOWNRUPT WORD1 and DOWNRUPT WORD2 addresses.
    /// Whichever order the code writes to the downrupt is reflected in the state
    /// vector.
    ///
    /// Assumption: Code does not write to DOWNRUPT2 before DOWNRUPT1 word
    ///
    pub fn write(&mut self, channel_idx: usize, value: u16) {
        let packet = generate_yaagc_packet(channel_idx, value);
        self.tx.send(packet).unwrap();
    }
}
