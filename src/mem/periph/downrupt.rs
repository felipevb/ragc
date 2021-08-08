use serde::{Deserialize, Serialize};
use zmq;

pub struct DownruptPeriph {
    state: [u16; 200],
    current_idx: usize,
    socket: zmq::Socket,
}

#[derive(Serialize, Deserialize, Debug)]
struct DownruptMessage {
    state: Vec<u16>,
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
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PUB).unwrap();
        let _res = socket.bind("tcp://127.0.0.1:81969");

        DownruptPeriph {
            state: [0; 200],
            current_idx: 0,
            socket: socket,
        }
    }

    ///
    /// Function is responsible for sending ZMQ messages of the current downrupt
    /// state vector. This is done via serde and zmq
    ///
    ///
    fn send_downrupt_state(&self) {
        let msg = DownruptMessage {
            state: self.state.to_vec(),
        };
        let data = serde_json::to_string(&msg).unwrap();
        self.socket.send(&data, 0).unwrap();
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
    pub fn write(&mut self, _channel_idx: usize, value: u16) {
        self.state[self.current_idx] = value;
        self.current_idx += 1;
        if self.current_idx >= 200 {
            self.current_idx = 0;
            self.send_downrupt_state();
        }
    }
}

#[cfg(test)]
mod downrupt_unittest {
    #[test]
    ///
    /// # Description
    ///
    /// Test that the downrupt peripherial functions properly. For the Downrupt
    /// peripherial to work, the following should happen
    ///   - After 200 Word writes to the Downrupt message, a ZMQ message is sent
    ///     out via the publisher.
    ///
    fn test_message_sent() {
        let mut drupt = super::DownruptPeriph::new();

        // Setup Listener to ensure a message has shown up after we test the
        // Downrupt actually sends a message.
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::SUB).unwrap();
        let mut _res = socket.set_subscribe(b"");

        // Connect to the peripherial ZMQ bus. This will allow us to subscribe
        // to downrupt messages that should come in every 2 seconds.
        match socket.connect("tcp://127.0.0.1:81969") {
            Err(_x) => {
                panic!("Unable to connect to peripherial");
            }
            _ => {}
        };

        // Induce a sleep to allow the threads for ZMQ to sync up and connect
        // properly so we can get Downrupt messages.
        std::thread::sleep(std::time::Duration::new(0, 1000000));

        // Generate a Downrupt Message on ZMQ by filling up the Downrupt state
        // table. A complete downrupt is sent over the course of 2 seconds. (2
        // words every 20ms). In reality, the message is sent over in 2 word
        // chunks in the radar.
        for i in 0..200 {
            drupt.write(0, i);
        }

        let mut msg = zmq::Message::new();
        match socket.recv(&mut msg, 0) {
            Ok(_x) => {
                let a = msg.as_str().unwrap();
                let _dmsg: super::DownruptMessage = serde_json::from_str(a).unwrap();
            }
            Err(_x) => {
                panic!("Unable to recv message from Downrupt peripherial.");
            }
        };
    }
}
