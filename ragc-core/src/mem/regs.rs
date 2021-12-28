use crate::{cpu, consts};
use crate::mem::AgcMemType;


use log::debug;

#[derive(Clone)]
pub struct AgcRegs {
    regs: [u16; 32],
    pub fbank: usize,
    pub ebank: usize,
}

impl AgcRegs {
    pub fn new() -> AgcRegs {
        AgcRegs {
            regs: [0; 32],
            fbank: 0,
            ebank: 0,
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.regs = [0; 32];
        self.fbank = 0;
        self.ebank = 0;
    }

    fn update_bank_registers(&mut self) {
        let evalue: u16 = ((self.ebank & 0x7) << 8) as u16;
        let fvalue: u16 = ((self.fbank & 0x1F) << 10) as u16;
        let bvalue: u16 = (evalue >> 8) | fvalue;
        self.regs[consts::cpu::REG_EB] = evalue;
        self.regs[consts::cpu::REG_FB] = fvalue;
        self.regs[consts::cpu::REG_BB] = bvalue;
        debug!(
            "Updating Bank Registers: {:x} | {:x} | {:x}",
            evalue, fvalue, bvalue
        );
    }
}

impl AgcMemType for AgcRegs {
    fn read(&self, _bank_idx: usize, bank_offset: usize) -> u16 {
        match bank_offset {
            consts::cpu::REG_A | consts::cpu::REG_Q => self.regs[bank_offset],
            consts::cpu::REG_Z => self.regs[bank_offset] & 0o7777,
            consts::cpu::REG_ZERO => 0o00000,
            _ => self.regs[bank_offset] & 0o77777,
        }
    }

    fn write(&mut self, _bank_idx: usize, bank_offset: usize, value: u16) {
        match bank_offset {
            // BB register contains the bank index for both the Erasable memory
            // and ROM Memory window banks. As such, both this and BB register needs to be
            // updated.
            consts::cpu::REG_BB => {
                self.ebank = (value & 0x7) as usize;
                self.fbank = ((value & 0x7C00) >> 10) as usize;
                self.update_bank_registers();
                return;
            }

            // EB register contains the bank index for the Erasable memory
            // window bank. As such, both this and BB register needs to be
            // updated.
            consts::cpu::REG_FB => {
                self.fbank = ((value & 0x7C00) >> 10) as usize;
                self.update_bank_registers();
                return;
            }

            // EB register contains the bank index for the Erasable memory
            // window bank. As such, both this and BB register needs to be
            // updated.
            consts::cpu::REG_EB => {
                self.ebank = ((value & 0x0700) >> 8) as usize;
                self.update_bank_registers();
                return;
            }

            // Per the documentation of the Z register, this register is a
            // 12-bit register. As such, no writes or reads should have any
            // values beyond 12 bits.
            consts::cpu::REG_Z => {
                self.regs[bank_offset] = value & 0o7777;
            }

            // Zero register is hardwired to be zero. If there is a write to
            // the zero register, we should atleast warn the user.
            consts::cpu::REG_ZERO => {
                return;
            }

            // All remaining registers are standard 15-bit registers.
            _ => {
                self.regs[bank_offset] = value & 0o77777;
            }
        }
        self.regs[bank_offset] = value;
    }
}

#[cfg(test)]
mod regs_unittests {
    use super::AgcMemType;
    use super::AgcRegs;
    use crate::consts;

    #[test]
    ///
    /// # Description
    ///
    /// The following unittest ensures we get a proper s15 write and read across
    /// all possible values of the register
    ///
    fn test_lregister_i15() {
        let mut regs = AgcRegs::new();
        for val in 0o00000..=0o77777 {
            regs.write(0, consts::cpu::REG_L, val);
            let retval = regs.read(0, consts::cpu::REG_L);
            assert_eq!(
                retval, val,
                "Failed in register comparision: e: {:o} | r: {:o}",
                val, retval
            );
        }
    }

    #[test]
    ///
    /// # Description
    ///
    /// The following unittest ensures we get a proper i16 write and read across
    /// all possible values of the register
    ///
    fn test_lregister_i16() {
        let mut regs = AgcRegs::new();
        for val in 0o100000..=0o177777 {
            regs.write(0, consts::cpu::REG_L, val);
            let retval = regs.read(0, consts::cpu::REG_L);
            assert_eq!(
                retval,
                val & 0o77777,
                "Failed in register comparision: e: {:o} | r: {:o}",
                val & 0o77777,
                retval
            );
        }
    }

    #[test]
    ///
    /// # Description
    ///
    /// The following unittest ensures we get a proper i16 write and read across
    /// all possible values of the register
    ///
    fn test_i16_registers() {
        let mut regs = AgcRegs::new();
        let target_regs = [consts::cpu::REG_A, consts::cpu::REG_Q];
        for reg_idx in target_regs.iter() {
            for val in 0o000000..=0o177777 {
                regs.write(0, *reg_idx, val);
                let retval = regs.read(0, *reg_idx);
                assert_eq!(
                    retval,
                    val & 0o177777,
                    "Failed in register comparision: e: {:o} | r: {:o}",
                    val & 0o177777,
                    retval
                );
            }
        }
    }

    #[test]
    ///
    /// # Description
    ///
    /// The following unittest ensures we get a proper values for writing to the
    /// index bank values.
    ///
    fn test_bb_register() {
        let mut regs = AgcRegs::new();

        for ram_idx in 0..consts::RAM_NUM_BANKS {
            let test_eb_value = (0o7 & ram_idx) << 8;
            for rom_idx in 0..consts::ROM_NUM_BANKS {
                let test_fb_value = ((0o37) & rom_idx) << 10;
                let test_bb_value = test_fb_value | (ram_idx & 0o7);

                //println!("{:o} | {:o} | {:o}", test_bb_value, test_eb_value, test_fb_value);
                regs.write(0, consts::cpu::REG_BB, test_bb_value as u16);
                assert_eq!(test_bb_value as u16, regs.read(0, consts::cpu::REG_BB), "");
                assert_eq!(test_eb_value as u16, regs.read(0, consts::cpu::REG_EB), "");
                assert_eq!(test_fb_value as u16, regs.read(0, consts::cpu::REG_FB), "");
            }
        }
    }

    #[test]
    ///
    /// # Description
    ///
    /// The following unittest ensures we get a proper values for writing to the
    /// index bank values.
    ///
    fn test_eb_register() {
        let mut regs = AgcRegs::new();

        regs.write(0, consts::cpu::REG_FB, 0o00000);

        for ram_idx in 0..consts::RAM_NUM_BANKS {
            let test_eb_value = (0o7 & ram_idx) << 8;
            let test_bb_value = ram_idx & 0o7;

            //println!("{:o} | {:o} | {:o}", test_bb_value, test_eb_value, test_fb_value);
            regs.write(0, consts::cpu::REG_EB, test_eb_value as u16);
            assert_eq!(test_bb_value as u16, regs.read(0, consts::cpu::REG_BB), "");
            assert_eq!(test_eb_value as u16, regs.read(0, consts::cpu::REG_EB), "");
        }
    }

    #[test]
    ///
    /// # Description
    ///
    /// The following unittest ensures we get a proper values for writing to the
    /// index bank values.
    ///
    fn test_fb_register() {
        let mut regs = AgcRegs::new();

        regs.write(0, consts::cpu::REG_FB, 0o00000);

        for rom_idx in 0..consts::ROM_NUM_BANKS {
            let test_fb_value = ((0o37) & rom_idx) << 10;
            let test_bb_value = test_fb_value;

            //println!("{:o} | {:o} | {:o}", test_bb_value, test_eb_value, test_fb_value);
            regs.write(0, consts::cpu::REG_FB, test_fb_value as u16);
            assert_eq!(test_bb_value as u16, regs.read(0, consts::cpu::REG_BB), "");
            assert_eq!(test_fb_value as u16, regs.read(0, consts::cpu::REG_FB), "");
        }
    }

    #[test]
    ///
    /// Checks the proper functionality of the Z register. This test is to
    /// ensure the Z register is properly reporting back as a 12 bit register
    ///
    fn test_z_register() {
        let mut regs = AgcRegs::new();
        for val in 0o00000..=0o77777 {
            regs.write(0, consts::cpu::REG_Z, val);
            let retval = regs.read(0, consts::cpu::REG_Z);
            assert_eq!(
                retval,
                val & 0o07777,
                "Failed in register comparision: e: {:o} | r: {:o}",
                val & 0o07777,
                retval
            );
        }
    }
}
