use super::AgcInst;
use crate::cpu::*;

pub trait AgcInterrupt {
    fn inhint(&mut self, inst: &AgcInst) -> bool;
    fn relint(&mut self, inst: &AgcInst) -> bool;
    fn edrupt(&mut self, inst: &AgcInst) -> bool;
    fn resume(&mut self, inst: &AgcInst) -> bool;
}

impl AgcInterrupt for AgcCpu {
    fn inhint(&mut self, _inst: &AgcInst) -> bool {
        self.gint = false;
        self.cycles = 1;
        true
    }

    fn relint(&mut self, _inst: &AgcInst) -> bool {
        self.gint = true;
        self.cycles = 1;
        true
    }

    fn edrupt(&mut self, _inst: &AgcInst) -> bool {
        self.cycles = 3;

        // Inhibits interrupts
        self.gint = false;

        // Loads the Z register into the ZRUPT register
        // TODO

        // TODO: Takes the next instruction from address 0
        //   which sounds like IR = *Addr(0)

        false
    }

    fn resume(&mut self, _inst: &AgcInst) -> bool {
        self.cycles = 2;

        let val = self.read(REG_PC_SHADOW) - 1;
        self.write(REG_PC, val);
        self.ir = self.read(REG_IR);
        self.idx_val = 0;

        // Re-enable interrupts
        self.gint = true;
        self.is_irupt = false;

        false
    }
}

#[cfg(test)]
mod interrupt_instr_unittests {
    #[test]
    fn test_hello() {}
}
