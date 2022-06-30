use heapless::spsc::{Consumer, Producer};
use ragc_core;
use crate::utils::{get_7seg, get_7seg_value};

pub struct DskyDisplay<'a> {
    digit: [u8; 15],
    noun: u16,
    verb: u16,
    prog: u16,
    proceed: u16,
    output_flags: u16,
    keypress: Consumer<'a, u16, 8>,
    keypress_val: u16,
    dsky_tx: Producer<'a, (usize, u16), 64>,
    flash_tx: Producer<'a, u16, 8>,

    last_dsalmout: u16,
    last_dskyval: u16,
}

impl<'a> DskyDisplay<'a> {
    pub fn new(
        keypress_rx: Consumer<'a, u16, 8>,
        dsky_tx: Producer<'a, (usize, u16), 64>,
        flash_tx: Producer<'a, u16, 8>,
    ) -> Self {
        Self {
            digit: [0; 15],
            noun: 0,
            verb: 0,
            prog: 0,
            keypress: keypress_rx,
            dsky_tx: dsky_tx,
            keypress_val: 0,
            proceed: 0o20000,
            flash_tx: flash_tx,
            output_flags: 0x0,
            last_dsalmout: 0x0,
            last_dskyval: 0x0,
        }
    }

    ///
    /// # Description
    ///
    /// The function parses the CHANNEL_DSKY IO channel value into multiple parts
    /// that is used to capture the display nibbles and rows in which it is being
    /// displayed on.
    ///
    /// # Return Value
    ///
    /// - (u8 a, bool b, u8 c, u8 d)
    ///   - `a` - Represents the rown in which the decoding is being performed on
    ///   - `b` - Represents the boolean flag for some rows which determines the
    ///           positive or negative display flags for a given DSKY value
    ///   - `c` - Upper nibble of a given value on the DSKY
    ///   - `d` - Lower nibble of a given value on the DSKY
    ///
    fn parse_fields(&self, val: u16) -> (u8, bool, u8, u8) {
        let a: u8 = ((val >> 11) & 0xF) as u8;
        let c: u8 = ((val >> 5) & 0x1F) as u8;
        let d: u8 = (val & 0x1F) as u8;
        let b: bool = if val & (1 << 10) == (1 << 10) {
            true
        } else {
            false
        };
        (a, b, c, d)
    }

    pub fn read_keypress(&self) -> u16 {
        //debug!("DSKY: Reading keypress: {:?}", self.keypress_val);
        self.keypress_val & 0x1F
    }

    pub fn set_channel_value(&mut self, channel_idx: usize, value: u16) {
        match channel_idx {
            0o13 => {
                if value & 0o01000 != 0o00000 {
                    self.output_flags |= 0o00400;
                } else {
                    self.output_flags &= 0o77377;
                }
                match self.flash_tx.enqueue(self.output_flags) {
                    Err(x) => {
                        //warn!("Unable to push to DSKY Flashing queue");
                    }
                    _ => {}
                }
            }
            0o163 => {
                self.output_flags = value;
                match self.flash_tx.enqueue(self.output_flags) {
                    Err(x) => {
                        //warn!("Unable to push to DSKY Flashing queue");
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn get_channel_value(&self, channel_idx: usize) -> u16 {
        match channel_idx {
            0o163 => self.output_flags & 0o1771,
            _ => 0o00000,
        }
    }

    pub fn read_proceed_flag(&self) -> u16 {
        self.proceed
    }

    ///
    /// # Description
    ///
    /// This function is to set additional flags that is being controlled via
    /// CHANNEL_DSALMOUT
    pub fn set_dsalmout_flags(&mut self, flags: u16) {
        if self.last_dsalmout != flags {
            //debug!("DSKY: Setting CHANNEL_DSALMOUT Flags: {}", flags);
            self.last_dsalmout = flags;
            self.dsky_tx
                .enqueue((0o11, flags))
                .unwrap();

            self.output_flags = (self.output_flags & 0o77607) | (flags & 0o00170);
            match self.flash_tx.enqueue(self.output_flags) {
                Err(_x) => {
                    //warn!("Unable to push to DSKY Flashing queue");
                }
                _ => {}
            }
        }
    }

    ///
    /// # Description
    ///
    /// This function is to handle row value 12 (decimal) for CHANNEL_DSKY which
    /// is different than all the other rows. This row handles specific light
    /// indicators within the DSKYr
    ///
    /// # Arguments
    ///
    /// - `flags` - Bitfield for row 12 which represents the specific indicators
    ///             for that row.
    ///
    pub fn set_adv_flags(&mut self, _flags: u16) {
        //println!("DSKY: Setting Adv Flags: {:x}", flags);
        //self.lamps = (self.lamps & 0xFF) | ((flags as u32) << 8);
    }

    ///
    /// # Description
    ///
    /// This function is to handle new CHANNEL_DSKY values being set by the AGC.
    /// The function will parse the value and handle the appropriate display to
    /// update.
    ///
    /// # Arguments
    ///
    /// - `val` - Value that was written via the CHANNEL_DSKY io port.
    ///
    pub fn set_channel_dsky_value(&mut self, val: u16) {
        if self.last_dskyval == val {
            return;
        }

        //println!("DSKY: Setting CHANNEL_DSKY Value: {:x}", val);
        self.last_dskyval = val;

        let r = self.dsky_tx.enqueue((0o10, val));
        let (a, _b, c, d) = self.parse_fields(val);
        match a {
            1 => {
                self.digit[13] = get_7seg(c);
                self.digit[14] = get_7seg(d);
            }
            2 => {
                self.digit[11] = get_7seg(c);
                self.digit[12] = get_7seg(d);
            }
            3 => {
                self.digit[9] = get_7seg(c);
                self.digit[10] = get_7seg(d);
            }
            4 => {
                self.digit[7] = get_7seg(c);
                self.digit[8] = get_7seg(d);
            }
            5 => {
                self.digit[5] = get_7seg(c);
                self.digit[6] = get_7seg(d);
            }
            6 => {
                self.digit[3] = get_7seg(c);
                self.digit[4] = get_7seg(d);
            }
            7 => {
                self.digit[1] = get_7seg(c);
                self.digit[2] = get_7seg(d);
            }
            8 => {
                self.digit[0] = get_7seg(d);
            }
            11 => {
                self.prog = get_7seg_value(c, d);
            }
            10 => {
                self.verb = get_7seg_value(c, d);
            }
            9 => {
                self.noun = get_7seg_value(c, d);
            }
            12 => {
                self.set_adv_flags(val & 0x7FF);
            }
            _ => {}
        };
        //debug!("DSKY: {:?}", self.digit);
    }
}

impl ragc_core::mem::periph::AgcIoPeriph for DskyDisplay<'_> {
    fn read(&self, channel_idx: usize) -> u16 {
        match channel_idx {
            ragc_core::consts::io::CHANNEL_MNKEYIN => self.read_keypress(),
            ragc_core::consts::io::CHANNEL_CHAN30 => 0o77777,
            ragc_core::consts::io::CHANNEL_CHAN31 => 0o77777,
            ragc_core::consts::io::CHANNEL_CHAN32 => self.read_proceed_flag(),
            ragc_core::consts::io::CHANNEL_CHAN33 => 0o77777,
            0o163 => self.get_channel_value(channel_idx),
            _ => 0o00000,
        }
    }

    fn write(&mut self, channel_idx: usize, value: u16) {
        match channel_idx {
            ragc_core::consts::io::CHANNEL_DSKY => {
                self.set_channel_dsky_value(value);
            }
            ragc_core::consts::io::CHANNEL_DSALMOUT => {
                self.set_dsalmout_flags(value);
            }
            ragc_core::consts::io::CHANNEL_CHAN13 => {
                self.set_channel_value(channel_idx, value);
            }
            0o163 => {
                self.set_channel_value(channel_idx, value);
            }
            _ => {}
        }
    }

    fn is_interrupt(&mut self) -> u16 {
        if self.keypress.len() > 0 {
            let val = self.keypress.dequeue().unwrap();
            //info!("DSKY: Recv'd {}", val);
            match val & 0o40000 {
                0o40000 => {
                    self.proceed = val & 0o37777;
                }
                _ => {
                    self.keypress_val = val;
                    if self.keypress_val == 0o22 {
                        let io_val = self.get_channel_value(0o163);
                        self.set_channel_value(0o163, io_val & !0o00200);
                    }
                }
            }
            (1 << ragc_core::consts::cpu::RUPT_KEY1) as u16
        } else {
            0
        }
    }
}
