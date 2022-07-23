use yaagc_protocol;
use heapless::spsc::Queue;

enum YaagcParserState {
    CommandByte1,
    CommandByte2,
    DataBytes,
}

const COMMAND_BYTE_1: u8 = 0xAA;
const COMMAND_BYTE_2: u8 = 0x55;

pub struct YaagcParser {
    msg_buf: [u8; 4],
    state: YaagcParserState,
    count: usize,
    msgs: Queue<(usize, u16), 16>
}

pub fn generate_packet(channel: usize, value: u16) -> [u8; 6] {
    let mut msg = [0; 6];
    msg[0] = COMMAND_BYTE_1;
    msg[1] = COMMAND_BYTE_2;
    msg[2..6].copy_from_slice(&yaagc_protocol::agc::generate_yaagc_packet(channel, value));
    msg
}

impl YaagcParser {
    pub fn new() -> Self {
        YaagcParser {
            msg_buf: [0u8; 4],
            state: YaagcParserState::CommandByte1,
            count: 0,
            msgs: Queue::new()
        }
    }

    pub fn reset(&mut self) {
        self.msg_buf = [0u8; 4];
        self.state = YaagcParserState::CommandByte1;
        self.count = 0;
    }

    pub fn push_bytes(&mut self, new_data: &[u8]) {
        for b in new_data.into_iter() {
            match self.state {
                YaagcParserState::CommandByte1 => {
                    if b == &COMMAND_BYTE_1 {
                        self.state = YaagcParserState::CommandByte2;
                    }
                },
                YaagcParserState::CommandByte2 => {
                    if b == &COMMAND_BYTE_2 {
                        self.state = YaagcParserState::DataBytes;
                        self.count = 0;
                    }
                    else {
                        self.reset();
                    }
                },
                YaagcParserState::DataBytes => {
                    self.msg_buf[self.count] = *b;
                    self.count = self.count + 1;
                    if self.count >= 4 {
                        match yaagc_protocol::agc::parse_yaagc_packet(self.msg_buf) {
                            Some((x, y)) => {
                                _ = self.msgs.enqueue((x as usize, y));
                            },
                            None => {}
                        };
                        self.reset();
                    }
                }
            }
        }
    }

    pub fn count(&self) -> usize { self.msgs.len() }

    pub fn pop_message(&mut self) -> Option<(usize, u16)> {
        if self.msgs.is_empty() {
            None
        }
        else {
            Some(self.msgs.dequeue().unwrap())
        }
    }


}

