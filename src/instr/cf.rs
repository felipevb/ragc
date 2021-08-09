use super::AgcInst;
use crate::cpu::*;

use log::{error, warn};

pub trait AgcControlFlow {
    fn tcf(&mut self, inst: &AgcInst) -> bool;
    fn bzf(&mut self, inst: &AgcInst) -> bool;
    fn bzmf(&mut self, inst: &AgcInst) -> bool;
    fn ccs(&mut self, inst: &AgcInst) -> bool;
    fn tc(&mut self, inst: &AgcInst) -> bool;
}

impl AgcControlFlow for AgcCpu {
    fn bzf(&mut self, inst: &AgcInst) -> bool {
        self.ec_flag = false;

        let a = self.read(REG_A);
        match a {
            // TODO: Fix to handle overflow handling in this. This needs to be
            // checked because if there is an overflow, it is not treated as
            // zero.
            0 | 0xFFFF => {
                let next_addr = inst.get_data_bits() & 0xFFF;
                if (next_addr & 0xC00) == 0x0 {
                    warn!("BZF jumping to non-fixed memory!");
                }

                self.write(REG_PC, next_addr);
                self.ir = self.read(next_addr as usize);
                self.cycles = 1;
                false
            }
            // Return
            _ => {
                self.cycles = 1; //2 - TODO: Look into why the yaAGC is not doing this;
                true
            }
        }
    }

    fn tcf(&mut self, inst: &AgcInst) -> bool {
        let next_addr = inst.get_data_bits();
        self.update_pc(next_addr);
        self.ec_flag = false;
        self.cycles = 1;
        false
    }

    fn bzmf(&mut self, inst: &AgcInst) -> bool {
        let k = inst.get_data_bits();
        match k & 0xC00 {
            0x000 => {
                error!("Invalid encoding for BZMF");
                return true;
            }
            _ => {}
        }

        let a = self.read_s16(REG_A);
        match a {
            _ if a > 0x0000 && a < 0x7FFF => {
                self.cycles = 1; //2 - TODO: Look into why the yaAGC is not doing this;
                true
            }
            _ => {
                self.write(REG_PC, k);
                self.ir = self.read(k as usize);
                self.cycles = 1;
                self.ec_flag = false;
                false
            }
        }
    }

    fn ccs(&mut self, inst: &AgcInst) -> bool {
        let pc = self.read(REG_PC);
        let k = inst.get_kaddr_ram();
        let mut a = self.read_s16(k);

        // Check the Value to see if we need to error out do to
        // addressing. If we do, then we got a problem
        match a {
            0o000000 => {
                self.write(REG_PC, pc + 1);
                self.ir = self.read((pc + 1) as usize);
                self.write(REG_A, 0);
            }
            0o177777 => {
                self.write(REG_PC, pc + 3);
                self.ir = self.read((pc + 3) as usize);
                self.write(REG_A, 0);
            }
            0o000001..=0o077777 => {
                //_ if a > 0x0000 && a < 0x4000 => {
                self.write(REG_PC, pc);
                self.ir = self.read(pc as usize);
                self.write(REG_A, a - 1);
            }
            0o100000..=0o177776 => {
                //_ if a >= 0xC000 && a < 0xFFFF => {
                self.write(REG_PC, pc + 2);
                self.ir = self.read((pc + 2) as usize);
                a = a ^ 0xFFFF;
                self.write(REG_A, a - 1);
            }
        };

        // This instruction handles editing of the K value if they are the
        // edit registers.
        self.check_editing(k);

        self.cycles = 2;
        false
    }

    fn tc(&mut self, inst: &AgcInst) -> bool {
        self.cycles = 1;

        let k = inst.get_data_bits();
        let pc = self.read(REG_PC);

        //self.ir = self.read(k as usize);
        //debug!("TCF: pc: {:x} | k: {:x} | ir: {:x}", pc, k, self.ir);
        //self.write(REG_PC, k);
        self.update_pc(k);

        self.write(REG_LR, pc);
        self.ec_flag = false;

        false
    }
}

#[cfg(test)]
mod cfg_tests {
    use crate::cpu;
    use crate::instr::tests::{init_agc, validate_cpu_state};

    ///
    /// ## CCS Absolute Value Test.
    ///
    /// This is performing a test that came up during debugging where the
    /// absolute value was not being performed properly. The following printout
    /// demonstrates the divergence.
    ///
    ///     PROCA: AgcInst { pc: 78a, mnem: CCS, inst_data: 1000, extrabits: Some(0), mct: 1 } | 078a | 078a
    ///     PROCB: CCS	0000
    ///         fc2e | 042b | 00de | 0000 | 1400 | 1400 | 078a | 0a79 ||| 40
    ///         fc2e | 042b | 00de | 0000 | 1400 | 1400 | 078a | 0a79 ||| 40
    ///     PROCA: AgcInst { pc: 78d, mnem: XCH, inst_data: 5cde, extrabits: Some(3), mct: 1 } | 078d | 078d
    ///     PROCB: XCH	0336
    ///         83d0 | 042b | 00de | 0000 | 1400 | 1400 | 078d | 0a79 ||| 46
    ///         03d0 | 042b | 00de | 0000 | 1400 | 1400 | 078d | 0a79 ||| 46
    ///
    ///
    #[test]
    fn ccs_test() {
        let mut cpu = init_agc();
        let inst_data: u16 = 0o10000;

        cpu.write(0x800, inst_data);
        cpu.reset();
        cpu.write(cpu::REG_A, 0xfc2e);
        assert_eq!(cpu.read(cpu::REG_A), 0xfc2e);
        cpu.step();

        validate_cpu_state(&cpu, 0x803);
        assert_eq!(cpu.read(cpu::REG_A), 0x03d0); // This should be |A|-1
    }
}