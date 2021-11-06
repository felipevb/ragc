use log::{info, warn};

use crate::mem::AgcMemType;

#[allow(dead_code)]
const DATA_LINE_NUM_PARTS: usize = 8;
#[allow(dead_code)]
const DATA_LINE_PART_LEN: usize = 6;
pub const ROM_BANKS_NUM: usize = 36;
pub const ROM_BANK_NUM_WORDS: usize = 1024;


enum Option<T> {
    None,
    Some(T)
}

pub struct AgcRom<'a> {
    program: Option<&'a [[u16; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM]>
}



// ============================================================================
// Trait Implementations
// ============================================================================
impl <'a>AgcMemType for AgcRom<'a> {
    fn read(&self, bank_idx: usize, bank_offset: usize) -> u16 {
        if bank_idx >= ROM_BANKS_NUM || bank_offset >= ROM_BANK_NUM_WORDS {
            warn!(
                "Out of bound indexing into AgcRom {} {}",
                bank_idx, bank_offset
            );
            return 0x0;
        }
        match self.program {
            Option::Some(program) => {
                const BANK_IDX_REF: [usize; 36] = [
                    2, 3, 0, 1, 4, 5, 6, 7,
                    8, 9, 10, 11, 12, 13, 14, 15,
                    16, 17, 18, 19, 20, 21, 22, 23,
                    24, 25, 26, 27, 28, 29, 30, 31,
                    32, 33, 34, 35,
                ];
                (u16::from_be(program[BANK_IDX_REF[bank_idx]][bank_offset]) >> 1) & 0x7FFF
            },
            _ => {
                0
            }
        }

    }

    fn write(&mut self, bank_idx: usize, bank_offset: usize, value: u16) {
        if bank_idx >= ROM_BANKS_NUM || bank_offset >= ROM_BANK_NUM_WORDS {
            warn!(
                "Out of bound indexing into AgcRom {} {}",
                bank_idx, bank_offset
            );
            return;
        }
        warn!("Attempting to write to AGC ROM. Ignoring write {:03o}{:03o} <= {:05o}",
                bank_idx, bank_offset, value);
    }
}

impl <'a>AgcRom<'a> {
    pub fn new(program: &'a [[u16; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM]) -> AgcRom {
        AgcRom {
            program: Option::Some(program),
        }
    }

    pub fn blank() -> AgcRom<'a> {
        AgcRom {
            program: Option::None
        }
    }

    #[allow(dead_code)]
    pub fn print_mem(&self) {
        match self.program {
            Option::Some(program) => {
                for (idx, b) in program.iter().enumerate() {
                    info!("Bank {}", idx);
                    for v in b.chunks(8) {
                        info!(
                            "\t{:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x}",
                            v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]
                        );
                    }
                }
            },
            Option::None => {
                info!("ROM is Blank.");
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
