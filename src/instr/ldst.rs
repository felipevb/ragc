use super::AgcInst;
use crate::cpu::*;
use crate::utils;

use log::debug;

pub trait AgcLoadStore {
    fn cs(&mut self, inst: &AgcInst) -> u16;
    fn ca(&mut self, inst: &AgcInst) -> u16;
    fn dcs(&mut self, inst: &AgcInst) -> u16;
    fn dca(&mut self, inst: &AgcInst) -> u16;
    fn xch(&mut self, inst: &AgcInst) -> u16;
    fn dxch(&mut self, inst: &AgcInst) -> u16;
    fn lxch(&mut self, inst: &AgcInst) -> u16;
    fn qxch(&mut self, inst: &AgcInst) -> u16;
    fn ts(&mut self, inst: &AgcInst) -> u16;
}

impl AgcLoadStore for AgcCpu {
    fn cs(&mut self, inst: &AgcInst) -> u16 {
        let addr: usize = inst.get_data_bits() as usize;
        let mut val = self.read_s16(addr);
        //debug!("Addr/Val: {:x?}/{:x?}", addr, val);

        val = !val;
        val = val & 0xFFFF;
        self.write_s16(REG_A, val);
        self.check_editing(inst.get_kaddr());
        2
    }

    fn dcs(&mut self, inst: &AgcInst) -> u16 {
        let k = inst.get_kaddr() - 1;

        let val_l = (!self.read_s16(k + 1)) & 0xFFFF;
        self.write(REG_L, val_l);

        let val_a = (!self.read_s16(k)) & 0xFFFF;
        self.write(REG_A, val_a);

        self.check_editing(k + 1);
        self.check_editing(k);

        3
    }

    fn dca(&mut self, inst: &AgcInst) -> u16 {
        // To handle the odd case of "DCA L" instruction, we wil break this
        // up into multiple load and stores just like the hardware handles it.
        // In essence, this loads Q into A and L
        let k = inst.get_kaddr() - 1;

        let val_l = self.read_s16(k + 1);
        self.write_s16(REG_L, val_l);

        let val_a = self.read_s16(k);
        self.write_s16(REG_A, val_a);

        self.check_editing(k + 1);
        self.check_editing(k);

        3
    }

    fn dxch(&mut self, inst: &AgcInst) -> u16 {
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
                let idx = self.read(REG_Z) as usize;
                self.ir = self.read(idx);
            }
            _ => {}
        }

        3
    }

    fn lxch(&mut self, inst: &AgcInst) -> u16 {
        let k = inst.get_kaddr_ram();

        let lval = self.read_s16(REG_L);
        let kval = self.read_s16(k);

        self.write_s16(REG_L, kval);
        self.write_s16(k, lval);

        2
    }

    fn ca(&mut self, inst: &AgcInst) -> u16 {
        let addr: usize = inst.get_data_bits() as usize;
        let val = self.read_s16(addr);
        self.write_s16(REG_A, val);
        self.check_editing(addr);
        2
    }

    fn ts(&mut self, inst: &AgcInst) -> u16 {
        let addr = inst.get_kaddr_ram();
        let a = self.read_s16(REG_A);

        match a & 0xC000 {
            // Negative Overflow Scenario
            0x8000 => {
                self.write_s16(REG_A, 0xFFFE);
                let val = self.read(REG_PC) + 1;
                self.update_pc(val);
            }
            // Positive Overflow Scenario
            0x4000 => {
                self.write_s16(REG_A, 0x0001);
                let val = self.read(REG_PC) + 1;
                self.update_pc(val);
            }
            _ => {}
        };

        self.write_s16(addr, a);
        self.read(addr);
        2
    }

    fn qxch(&mut self, inst: &AgcInst) -> u16 {
        let k = inst.get_kaddr_ram();
        let v = self.read_s16(k as usize);
        let v_q = self.read_s16(REG_LR);

        self.write_s16(k as usize, v_q);
        self.write_s16(REG_LR, v);
        2
    }

    fn xch(&mut self, inst: &AgcInst) -> u16 {
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
        2
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
        validate_cpu_state(&mut cpu, 0x802);
        assert_eq!(cpu.read(cpu::REG_A), 0x0001);

        cpu.step();
        validate_cpu_state(&mut cpu, 0x803);
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
