use yaagc_protocol::agc::{generate_yaagc_packet, parse_yaagc_packet};
use crate::utils::{get_7seg, get_7seg_value};

use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{debug, warn};

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::println;

pub struct DskyDisplay {
    digit: [u8; 15],
    noun: u16,
    verb: u16,
    prog: u16,
    proceed: u16,
    output_flags: u16,
    keypress: Receiver<u16>,
    keypress_val: u16,
    dsky_tx: Sender<[u8; 4]>,
    flash_tx: Sender<u16>,
    last_dsalmout: u16,
    last_dskyval: u16,
}

fn handle_stream_input(stream: &mut TcpStream, keypress_tx: &Sender<u16>) {
    loop {
        let mut buf = [0; 4];
        match stream.read_exact(&mut buf) {
            Ok(_x) => match parse_yaagc_packet(buf) {
                Some(res) => match res.0 {
                    0o15 => {
                        debug!("Keypress: {:o}", res.1);
                        let _res = keypress_tx.send(res.1);
                    }
                    0o32 => {
                        debug!("Keypress (Proceed): {:o}", res.1);
                        let _res = keypress_tx.send(res.1 | 0o40000);
                    }
                    _ => {
                        warn!("Unimplemented keypress: {:?}", res);
                    }
                },
                _ => {}
            },
            _ => {
                break;
            }
        }
    }
    println!("Stream Input: Disconnecting from stream session");
}

fn handle_steam_output(stream: &mut TcpStream, dsky_rx: &Receiver<[u8; 4]>) {
    loop {
        let msg = match dsky_rx.recv() {
            Ok(x) => x,
            _ => {
                break;
            }
        };

        match stream.write_all(&msg) {
            Ok(_x) => {}
            _ => {
                break;
            }
        }
    }
    println!("Stream Output: Disconnecting from stream session");
}

fn flashing_thread(flash_rx: Receiver<u16>, dsky_tx: Sender<[u8; 4]>) {
    let mut channel_value = 0o00000;
    let start_time = std::time::SystemTime::now();

    loop {
        while !flash_rx.is_empty() {
            channel_value = flash_rx.recv().unwrap();
        }

        let elapsed = start_time.elapsed().unwrap().as_millis();
        if elapsed % 1000 < 750 {
            let mut value = channel_value;
            if channel_value & 0o00040 == 0o00040 {
                value &= !0o00040;
            }
            dsky_tx.send(generate_yaagc_packet(0o0163, value)).unwrap();
        } else {
            if channel_value != 0o00000 {
                let mut value = channel_value & !0o00160;
                if channel_value & 0o00040 == 0o00040 {
                    value |= 0o00040;
                }
                dsky_tx.send(generate_yaagc_packet(0o0163, value)).unwrap();
            }
        }

        std::thread::sleep(std::time::Duration::new(0, 10000000));
    }
}

fn dsky_network_thread(keypress_tx: Sender<u16>, dsky_rx: Receiver<[u8; 4]>) {
    // accept connections and process them serially
    let listener = TcpListener::bind("127.0.0.1:19697").unwrap();
    for stream in listener.incoming() {
        println!("Connecting to new stream");
        match stream {
            Ok(mut xa) => {
                match xa.try_clone() {
                    Ok(mut x) => {
                        let keypresstx = keypress_tx.clone();
                        std::thread::spawn(move || handle_stream_input(&mut x, &keypresstx));
                    }
                    _ => {
                        continue;
                    }
                }
                handle_steam_output(&mut xa, &dsky_rx);
            }
            _ => {}
        };
        println!("Disconnecting");
    }
}

impl DskyDisplay {
    pub fn new() -> Self {
        let (keypress_tx, keypress_rx) = unbounded();
        let (dsky_tx, dsky_rx) = unbounded();
        let (flash_tx, flash_rx) = unbounded();

        let flash_dsky_tx = dsky_tx.clone();
        std::thread::spawn(move || flashing_thread(flash_rx, flash_dsky_tx));
        std::thread::spawn(move || dsky_network_thread(keypress_tx, dsky_rx));

        Self {
            digit: [0; 15],
            noun: 0,
            verb: 0,
            prog: 0,
            keypress: keypress_rx,
            keypress_val: 0,
            proceed: 0o20000,
            dsky_tx: dsky_tx,
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
        debug!("DSKY: Reading keypress: {:?}", self.keypress_val);
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
                self.flash_tx.send(self.output_flags).unwrap();
            }
            0o163 => {
                self.output_flags = value;
                self.flash_tx.send(self.output_flags).unwrap();
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
            debug!("DSKY: Setting CHANNEL_DSALMOUT Flags: {:o}", flags);
            self.last_dsalmout = flags;
            self.dsky_tx
                .send(generate_yaagc_packet(0o11, flags))
                .unwrap();

            self.output_flags = (self.output_flags & 0o77607) | (flags & 0o00170);
            self.flash_tx.send(self.output_flags).unwrap();
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
        self.dsky_tx.send(generate_yaagc_packet(0o10, val)).unwrap();

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
        debug!("DSKY: {:?}", self.digit);
    }
}

impl ragc_core::mem::periph::AgcIoPeriph for DskyDisplay {
    fn read(&self, channel_idx: usize) -> u16 {
        match channel_idx {
            ragc_core::consts::io::CHANNEL_MNKEYIN => {
                self.read_keypress()
            }
            ragc_core::consts::io::CHANNEL_CHAN30 => 0o77777,
            ragc_core::consts::io::CHANNEL_CHAN31 => 0o77777,
            ragc_core::consts::io::CHANNEL_CHAN32 => {
                self.read_proceed_flag()
            }
            ragc_core::consts::io::CHANNEL_CHAN33 => 0o77777,
            0o163 => self.get_channel_value(channel_idx),
            _ => { 0o00000 }
        }
    }

    fn write(&mut self, channel_idx: usize, value: u16) {
        match channel_idx {
            ragc_core::consts::io::CHANNEL_DSKY => {
                self.set_channel_dsky_value(value);
            },
            ragc_core::consts::io::CHANNEL_DSALMOUT => {
                self.set_dsalmout_flags(value);
            }
            ragc_core::consts::io::CHANNEL_CHAN13 => {
                self.set_channel_value(channel_idx, value);
            }
            0o163 => {
                self.set_channel_value(channel_idx, value);
            }
            _ => { }
        }

    }

    fn is_interrupt(&mut self) -> u16 {
        if self.keypress.len() > 0 {
            let val = self.keypress.recv().unwrap();
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

#[cfg(test)]
mod dsky_unittests {
    use super::DskyDisplay;

    const AGC_SEG_TABLE: [u8; 11] = [
        // 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, Blank
        21, 3, 25, 27, 15, 30, 28, 19, 29, 31, 0,
    ];

    ///
    /// # Description
    ///
    /// # Arguments
    ///
    ///  -  `row_idx` - `u16` - Row within the CHANNEL_OUT0 encoding to update
    ///     a given digit nibble
    ///  - `digit_idx` - `usize` - Index into the `DskyState.digits` array in
    ///    which to compare the values
    ///
    fn dsky_display_digit_index(
        dsky: &mut DskyDisplay,
        row_idx: u16,
        lower_digit_seg: &u8,
        upper_digit_seg: &u8,
    ) {
        let value: u16 =
            (row_idx << 11) | *lower_digit_seg as u16 | ((*upper_digit_seg as u16) << 5);
        dsky.set_channel_dsky_value(value);
    }

    #[test]
    fn test_dsky_display_flags_yaagc() {
        let mut dsky = super::DskyDisplay::new();

        let mut value: u16 = (10 << 11) | 0o3 | 0o3;
        dsky.set_channel_dsky_value(value);
        value = (9 << 11) | 0o3 | 0o3;
        dsky.set_channel_dsky_value(value);

        for i in 0..8 {
            dsky.set_dsalmout_flags(1 << i);
            std::thread::sleep(std::time::Duration::new(5, 0));
            dsky.set_dsalmout_flags(0o0);
            std::thread::sleep(std::time::Duration::new(5, 0));
        }
    }

    #[test]
    fn test_dsky_display_flashing_yaagc() {
        let mut dsky = super::DskyDisplay::new();
        std::thread::sleep(std::time::Duration::new(5, 0));
        dsky_display_digit_index(
            &mut dsky,
            10,
            AGC_SEG_TABLE.get(7).unwrap(),
            AGC_SEG_TABLE.get(7).unwrap(),
        );
        dsky_display_digit_index(
            &mut dsky,
            9,
            AGC_SEG_TABLE.get(7).unwrap(),
            AGC_SEG_TABLE.get(7).unwrap(),
        );
        let mut val = 0o00000;
        loop {
            val ^= 0o01771;
            dsky.dsky_tx
                .send(super::generate_yaagc_packet(0o0163, val))
                .unwrap();
            std::thread::sleep(std::time::Duration::new(1, 0));
        }
    }

    #[test]
    fn test_dsky_display_yaagc() {
        let mut dsky = super::DskyDisplay::new();

        for upper_digit_seg in AGC_SEG_TABLE.iter() {
            for lower_digit_seg in AGC_SEG_TABLE.iter() {
                let value: u16 =
                    (11 << 11) | *lower_digit_seg as u16 | ((*upper_digit_seg as u16) << 5);
                dsky.set_channel_dsky_value(value);

                let value: u16 =
                    (10 << 11) | *lower_digit_seg as u16 | ((*upper_digit_seg as u16) << 5);
                dsky.set_channel_dsky_value(value);

                let value: u16 =
                    (9 << 11) | *lower_digit_seg as u16 | ((*upper_digit_seg as u16) << 5);
                dsky.set_channel_dsky_value(value);

                dsky_display_digit_index(&mut dsky, 8, upper_digit_seg, &0);
                dsky_display_digit_index(&mut dsky, 7, lower_digit_seg, upper_digit_seg);
                dsky_display_digit_index(&mut dsky, 6, lower_digit_seg, upper_digit_seg);
                dsky_display_digit_index(&mut dsky, 5, lower_digit_seg, upper_digit_seg);
                dsky_display_digit_index(&mut dsky, 4, lower_digit_seg, upper_digit_seg);
                dsky_display_digit_index(&mut dsky, 3, lower_digit_seg, upper_digit_seg);
                dsky_display_digit_index(&mut dsky, 2, lower_digit_seg, upper_digit_seg);
                dsky_display_digit_index(&mut dsky, 1, lower_digit_seg, upper_digit_seg);

                let dur = std::time::Duration::new(0, 100000000);
                std::thread::sleep(dur);
            }
        }
    }
}
