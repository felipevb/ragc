use log::{error, info, warn};
use std::fs::File;
use std::io::Read;

use crate::mem::AgcMemType;

#[allow(dead_code)]
const DATA_LINE_NUM_PARTS: usize = 8;
#[allow(dead_code)]
const DATA_LINE_PART_LEN: usize = 6;
pub const ROM_BANKS_NUM: usize = 36;
pub const ROM_BANK_NUM_WORDS: usize = 1024;

#[derive(Clone)]
pub struct AgcRom {
    //filepath: String,
    bank_idx: u8,
    line_cnt: u8,
    parity: u8,
    banks: [[u16; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM],
}

// ============================================================================
// Trait Implementations
// ============================================================================
impl AgcMemType for AgcRom {
    fn read(&self, bank_idx: usize, bank_offset: usize) -> u16 {
        if bank_idx >= ROM_BANKS_NUM || bank_offset >= ROM_BANK_NUM_WORDS {
            warn!(
                "Out of bound indexing into AgcRom {} {}",
                bank_idx, bank_offset
            );
            return 0x0;
        }
        self.banks[bank_idx][bank_offset] & 0x7FFF
    }

    fn write(&mut self, bank_idx: usize, bank_offset: usize, value: u16) {
        if bank_idx >= ROM_BANKS_NUM || bank_offset >= ROM_BANK_NUM_WORDS {
            warn!(
                "Out of bound indexing into AgcRom {} {}",
                bank_idx, bank_offset
            );
            return;
        }
        self.banks[bank_idx][bank_offset] = value
    }
}

impl AgcRom {
    pub fn new() -> AgcRom {
        AgcRom {
            bank_idx: 0,
            line_cnt: 0,
            parity: 0,
            banks: [[0; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM],
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.banks = [[0; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM];
    }

    pub fn load_agcbin_file(&mut self, filename: &str) -> Option<()> {
        // Check to make sure we are able to open the file. If we are not
        // able to, throw up the issue up to the caller to know we failed
        // at opening the file.
        let fp = File::open(filename);
        let mut f = match fp {
            Ok(f) => f,
            _ => {
                error!("Unable to open file: {:?}", filename);
                return None;
            }
        };

        let mut buf = [0; ROM_BANK_NUM_WORDS * 2];
        let bank_idx_ref = [
            2, 3, 0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
        ];

        let mut bank_idx = 0;
        loop {
            match f.read_exact(&mut buf) {
                Ok(_x) => {
                    let mut word_idx = 0;
                    for c in buf.chunks_exact(2) {
                        let res = (c[0] as u16) << 8 | c[1] as u16;
                        self.banks[bank_idx_ref[bank_idx]][word_idx] = res >> 1;
                        word_idx += 1;
                    }
                }
                Err(_x) => {
                    break;
                }
            };
            bank_idx += 1;
        }

        None
    }

    #[allow(dead_code)]
    pub fn print_mem(&self) {
        for (idx, b) in self.banks.iter().enumerate() {
            info!("Bank {}", idx);
            for v in b.chunks(8) {
                info!(
                    "\t{:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x}",
                    v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]
                );
            }
        }
    }
}

// ============================================================================
// Private Functions
// ============================================================================

// ============================================================================
// Module Tests
// ============================================================================
#[cfg(test)]
mod agc_rom_tests {
    use super::*;

    #[test]
    fn reset_test() {
        let mut rom = AgcRom::new();

        // Load with Random Value to ensure reset will do what it should be
        // doing.
        for i in 0..ROM_BANKS_NUM {
            for j in 0..ROM_BANK_NUM_WORDS {
                rom.banks[i][j] = 0xAA55;
            }
        }

        // Reset
        rom.reset();
        for i in 0..ROM_BANKS_NUM {
            for j in 0..ROM_BANK_NUM_WORDS {
                assert_eq!(0, rom.banks[i][j]);
            }
        }
    }

    #[test]
    fn read_test() {
        let mut rom = AgcRom::new();
        for i in 0..ROM_BANKS_NUM {
            for j in 0..ROM_BANK_NUM_WORDS {
                rom.reset();
                rom.banks[i][j] = 0x55AA;
                assert_eq!(
                    0x55AA,
                    rom.read(i, j),
                    "Failed reading Bank {:?} Offset {:?}",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn write_test() {
        let mut rom = AgcRom::new();

        for i in 0..ROM_BANKS_NUM {
            for j in 0..ROM_BANK_NUM_WORDS {
                rom.reset();
                rom.write(i, j, 0x55AA);
                assert_eq!(
                    0x55AA, rom.banks[i][j],
                    "Failed Writing @ Bank {:?} Offset {:?}",
                    i, j
                );
            }
        }
    }
}
