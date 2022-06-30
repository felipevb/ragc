use ragc_core;
use ragc_core::mem::periph::AgcIoPeriph;
use heapless::spsc::Producer;

pub struct DownruptPeriph<'a> {
    //tx: Sender<[u8; 4]>,
    tx: Producer<'a, (usize, u16), 4>,
    word_order: bool,
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
impl<'a> DownruptPeriph<'a> {
    pub fn new(downrupt_prod: Producer<'a, (usize, u16), 4>) -> Self {
        DownruptPeriph {
            tx: downrupt_prod,
            word_order: false,
        }
    }
}

impl AgcIoPeriph for DownruptPeriph<'_> {
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
                } else {
                    0o00000
                }
            }
            ragc_core::consts::io::CHANNEL_CHAN30
            | ragc_core::consts::io::CHANNEL_CHAN31
            | ragc_core::consts::io::CHANNEL_CHAN32
            | ragc_core::consts::io::CHANNEL_CHAN33
            | ragc_core::consts::io::CHANNEL_CHAN34
            | ragc_core::consts::io::CHANNEL_CHAN35 => 0o77777,
            _ => 0o00000,
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
                } else {
                    self.word_order = false;
                }
            }
            ragc_core::consts::io::CHANNEL_CHAN34 => {
                let packet = (channel_idx, value);
                let r = self.tx.enqueue(packet).unwrap();
                //info!("Done Writing CHAN34");
            }
            ragc_core::consts::io::CHANNEL_CHAN35 => {
                let packet = (channel_idx, value);
                self.tx.enqueue(packet).unwrap();
                //info!("Done Writing CHAN35");
            }
            _ => {}
        }
    }

    fn is_interrupt(&mut self) -> u16 {
        0
    }
}
