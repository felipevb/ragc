use crate::cpu;
use crate::mem::AgcMemType;
use log::trace;

/* Number of Banks within a given AGC computer */
pub const RAM_NUM_BANKS: usize = 8;

/* Number of Words within a given RAM Bank */
const RAM_BANK_SIZE: usize = 256;

#[derive(Clone)]
pub struct AgcRam {
    banks: [[u16; RAM_BANK_SIZE]; RAM_NUM_BANKS],
}

impl AgcRam {
    ///
    /// Constructor for AgcRam structure. This will create a blank RAM state of
    /// all zeros within a given RAM structure.
    ///
    pub fn new() -> AgcRam {
        AgcRam {
            banks: [[0; RAM_BANK_SIZE]; RAM_NUM_BANKS],
        }
    }

    ///
    /// Function will perform a reset on the RAM and set the contents of RAM to
    /// zero. This is only done when really needed. Actual AGC RAM actually
    /// retained its contents across resets unless it was actively set to zero
    ///
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.banks = [[0; RAM_BANK_SIZE]; RAM_NUM_BANKS];
    }
}

impl AgcMemType for AgcRam {
    ///
    /// AgcRam implementation of `read` function. This function will handle U16
    /// read requests to a given memory RAM location. All addresses are word
    /// addresses.
    ///
    /// # Arguments
    ///
    ///  - `bank_idx` - usize - Bank index of RAM to write to. For the AGC, there
    ///    are multiple RAM banks that are used.
    ///  - `bank_offset` - usize - Word offset within a given `bank_idx`
    ///
    /// # Return Value
    ///
    ///  - `value` - u16 - Value that is located within the given `bank_idx`
    ///    and `bank_offset`
    ///
    fn read(&self, bank_idx: usize, bank_offset: usize) -> u16 {
        let res = if bank_idx == 0x0 && bank_offset == cpu::REG_A {
            self.banks[bank_idx][bank_offset]
        } else if bank_idx == 0x0 && bank_offset == cpu::REG_Q {
            self.banks[bank_idx][bank_offset]
        } else {
            self.banks[bank_idx][bank_offset] & 0x7FFF
        };
        trace!(
            "RAM Read: 0x{:x},0x{:x}: 0x{:x}",
            bank_idx,
            bank_offset,
            res
        );
        res
    }

    ///
    /// AgcRam implementation of `write` function. This function will handle U16
    /// writes to a given memory RAM location. All addresses are word addresses.
    ///
    /// # Arguments
    ///
    ///  - `bank_idx` - usize - Bank index of RAM to write to. For the AGC, there
    ///    are multiple RAM banks that are used.
    ///  - `bank_offset` - usize - Word offset within a given `bank_idx`
    ///  - `value` - u16 - Value to write to a given RAM address.
    ///
    fn write(&mut self, bank_idx: usize, bank_offset: usize, value: u16) {
        trace!(
            "RAM Write: 0x{:x},0x{:x}: 0x{:x}",
            bank_idx,
            bank_offset,
            value
        );
        if bank_idx == 0x0 && bank_offset == cpu::REG_A {
            self.banks[bank_idx][bank_offset] = value;
        } else if bank_idx == 0x0 && bank_offset == cpu::REG_Q {
            self.banks[bank_idx][bank_offset] = value;
        } else {
            let a = value & 0x7FFF;
            self.banks[bank_idx][bank_offset] = a;
        }
    }
}

#[cfg(test)]
mod agc_ram_tests {
    use super::*;

    #[test]
    fn reset_test() {
        let mut ram = AgcRam::new();

        // Load with Random Value to ensure reset will do what it should be
        // doing.
        for i in 0..RAM_NUM_BANKS {
            for j in 0..RAM_BANK_SIZE {
                ram.banks[i][j] = 0xAA55;
            }
        }

        // Reset
        ram.reset();
        for i in 0..RAM_NUM_BANKS {
            for j in 0..RAM_BANK_SIZE {
                assert_eq!(0, ram.banks[i][j]);
            }
        }
    }

    #[test]
    fn test_read_s15_locations() {
        let mut ram = AgcRam::new();

        for i in 0..RAM_NUM_BANKS {
            for j in 0..RAM_BANK_SIZE {
                ram.reset();
                ram.banks[i][j] = 0x55AA;
                assert_eq!(
                    0x55AA,
                    ram.read(i, j),
                    "Failed reading Bank {:?} Offset {:?}",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn test_read_s16_locations() {
        let mut ram = AgcRam::new();
        let regs_16bit = [cpu::REG_A, cpu::REG_Q];

        // Testing 16Bit
        for reg_idx in regs_16bit.iter() {
            ram.reset();
            ram.banks[0][*reg_idx] = 0xFFFF;
            assert_eq!(
                0xFFFF,
                ram.read(0, *reg_idx),
                "Failed reading Bank {:?} Offset {:?}",
                0,
                *reg_idx
            );
        }

        // Test 15-Bit
        for i in 0..RAM_NUM_BANKS {
            for j in 0..RAM_BANK_SIZE {
                if i == 0 && regs_16bit.contains(&j) {
                    continue;
                }

                ram.reset();
                ram.banks[i][j] = 0xFFFF;
                assert_eq!(
                    0x7FFF,
                    ram.read(i, j),
                    "Failed reading Bank {:?} Offset {:?}",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn test_write_s15_locations() {
        let mut ram = AgcRam::new();

        for i in 0..RAM_NUM_BANKS {
            for j in 0..RAM_BANK_SIZE {
                ram.reset();
                ram.write(i, j, 0x55AA);
                assert_eq!(
                    0x55AA, ram.banks[i][j],
                    "Failed Writing @ Bank {:?} Offset {:?}",
                    i, j
                );
            }
        }
    }

    #[test]
    fn test_write_s16_locations() {
        let mut ram = AgcRam::new();
        let regs_16bit = [cpu::REG_A, cpu::REG_Q];

        // Testing 16Bit
        for reg_idx in regs_16bit.iter() {
            ram.reset();
            ram.write(0, *reg_idx, 0xFFFF);
            assert_eq!(
                0xFFFF, ram.banks[0][*reg_idx],
                "Failed Writing @ Bank {:?} Offset {:?}",
                0, *reg_idx
            );
        }
    }
}
