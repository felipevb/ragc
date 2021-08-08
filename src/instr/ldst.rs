use super::AgcInst;
use crate::cpu::*;
use crate::utils;

use log::debug;

pub trait AgcLoadStore {
    fn cs(&mut self, inst: &AgcInst) -> bool;
    fn ca(&mut self, inst: &AgcInst) -> bool;
    fn dcs(&mut self, inst: &AgcInst) -> bool;
    fn dca(&mut self, inst: &AgcInst) -> bool;
    fn xch(&mut self, inst: &AgcInst) -> bool;
    fn dxch(&mut self, inst: &AgcInst) -> bool;
    fn lxch(&mut self, inst: &AgcInst) -> bool;
    fn qxch(&mut self, inst: &AgcInst) -> bool;
    fn ts(&mut self, inst: &AgcInst) -> bool;
}

impl AgcLoadStore for AgcCpu {
    fn cs(&mut self, inst: &AgcInst) -> bool {
        self.cycles = 2;

        let addr: usize = inst.get_data_bits() as usize;
        let mut val = self.read_s16(addr);
        //debug!("Addr/Val: {:x?}/{:x?}", addr, val);

        val = !val;
        val = val & 0xFFFF;
        self.write_s16(REG_A, val);
        self.check_editing(inst.get_kaddr());
        true
    }

    fn dcs(&mut self, inst: &AgcInst) -> bool {
        self.cycles = 3;

        let k = inst.get_kaddr() - 1;
        self.write(REG_L, (!self.read_s16(k + 1)) & 0xFFFF);
        self.write(REG_A, (!self.read_s16(k)) & 0xFFFF);

        self.check_editing(k + 1);
        self.check_editing(k);

        true
    }

    fn dca(&mut self, inst: &AgcInst) -> bool {
        self.cycles = 3;

        // To handle the odd case of "DCA L" instruction, we wil break this
        // up into multiple load and stores just like the hardware handles it.
        // In essence, this loads Q into A and L
        let k = inst.get_kaddr() - 1;
        self.write_s16(REG_L, self.read_s16(k + 1));
        self.write_s16(REG_A, self.read_s16(k));

        self.check_editing(k + 1);
        self.check_editing(k);

        true
    }

    fn dxch(&mut self, inst: &AgcInst) -> bool {
        self.cycles = 3;

        let kaddr = inst.get_kaddr_ram() - 1;

        let l = self.read_s16(REG_L);
        let k2 = self.read_s16(kaddr + 1);
        self.write_s16(REG_L, k2);
        self.write_s16(kaddr + 1, l);

        let a = self.read_s16(REG_A);
        let k1 = self.read_s16(kaddr);
        self.write_s16(REG_A, k1);
        self.write_s16(kaddr, a);

        match inst.get_kaddr_ram() {
            5 | 6 => {
                self.ir = self.read(self.read(REG_Z) as usize);
            }
            _ => {}
        }

        true
    }

    fn lxch(&mut self, inst: &AgcInst) -> bool {
        self.cycles = 2;

        let k = inst.get_kaddr_ram();

        let lval = self.read_s16(REG_L);
        let kval = self.read_s16(k);

        self.write_s16(REG_L, kval);
        self.write_s16(k, lval);

        false
    }

    fn ca(&mut self, inst: &AgcInst) -> bool {
        self.cycles = 2;

        let addr: usize = inst.get_data_bits() as usize;
        let val = self.read_s16(addr);
        self.write_s16(REG_A, val);
        self.check_editing(addr);
        true
    }

    fn ts(&mut self, inst: &AgcInst) -> bool {
        self.cycles = 2;
        let addr = inst.get_kaddr_ram();
        let a = self.read_s16(REG_A);

        match a & 0xC000 {
            // Negative Overflow Scenario
            0x8000 => {
                self.write_s16(REG_A, 0xFFFE);
                //self.write(REG_PC, self.read(REG_PC) + 1);
                self.update_pc(self.read(REG_PC) + 1);
            }
            // Positive Overflow Scenario
            0x4000 => {
                self.write_s16(REG_A, 0x0001);
                //self.write(REG_PC, self.read(REG_PC) + 1);
                self.update_pc(self.read(REG_PC) + 1);
            }
            _ => {}
        };

        self.write_s16(addr, a);
        self.read(addr);
        true
    }

    fn qxch(&mut self, inst: &AgcInst) -> bool {
        self.cycles = 2;

        let k = inst.get_kaddr_ram();
        let v = self.read_s16(k as usize);
        let v_q = self.read_s16(REG_LR);

        self.write_s16(k as usize, v_q);
        self.write_s16(REG_LR, v);
        true
    }

    fn xch(&mut self, inst: &AgcInst) -> bool {
        self.cycles = 2;

        let k = inst.get_kaddr_ram();
        let v = self.read_s16(k);
        let v_q = self.read_s16(REG_A);

        debug!(
            "XCH: {:x} {:x} {:x}",
            k,
            utils::sign_extend(v),
            utils::overflow_correction(v_q)
        );
        self.write_s16(k, utils::overflow_correction(v_q));
        self.write_s16(REG_A, v);
        true
    }
}

#[cfg(test)]
mod ldst_tests {
    use crate::cpu;
    use crate::instr::tests::{init_agc, validate_cpu_state};

    #[test]
    fn ts_test_positive_ovsk() {
        let mut cpu = init_agc();

        cpu.write(0x800, 0o54000 + 0o100); // TS 0o100
        cpu.write(0x801, 0o30000 + 0o200); // CA 0o200
        cpu.write(0x802, 0o30000 + 0o300); // CA 0o300
        cpu.reset();

        cpu.write(cpu::REG_A, 0x4000);
        cpu.write(0o300, 0x00AA);
        cpu.write(0o200, 0x00BB);

        cpu.step();
        validate_cpu_state(&cpu, 0x802);
        assert_eq!(cpu.read(cpu::REG_A), 0x0001);

        cpu.step();
        validate_cpu_state(&cpu, 0x803);
        assert_eq!(cpu.read(cpu::REG_A), 0x00AA);
    }

    #[test]
    fn tests_lxch_no_overflow() {
        use super::AgcLoadStore;

        let mut cpu = init_agc();
        let mut inst = super::AgcInst::new();

        // Set the K Address that we would want to swap with
        // in order for this to work
        inst.inst_data = 0o00000; // We are swapping with REG_A to start
        cpu.write_s16(super::REG_A, 0o000001);
        cpu.write_s16(super::REG_L, 0o177777);
        assert_eq!(0o000001, cpu.read(super::REG_A));
        assert_eq!(0o077777, cpu.read(super::REG_L));

        cpu.lxch(&inst);

        assert_eq!(0o000001, cpu.read(super::REG_L));
        assert_eq!(0o177777, cpu.read(super::REG_A));
    }

    #[test]
    fn tests_lxch_with_overflow() {
        use super::AgcLoadStore;

        let mut cpu = init_agc();
        let mut inst = super::AgcInst::new();

        // Set the K Address that we would want to swap with
        // in order for this to work
        inst.inst_data = 0o00000; // We are swapping with REG_A to start
        cpu.write_s16(super::REG_A, 0o137777);
        cpu.write_s16(super::REG_L, 0o000001);
        assert_eq!(0o137777, cpu.read(super::REG_A));
        assert_eq!(0o000001, cpu.read(super::REG_L));

        cpu.lxch(&inst);

        assert_eq!(0o000001, cpu.read(super::REG_A));
        assert_eq!(0o077777, cpu.read(super::REG_L));
    }
}
