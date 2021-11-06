use super::AgcInst;
use crate::cpu::*;

pub trait AgcInterrupt {
    fn inhint(&mut self, inst: &AgcInst) -> u16;
    fn relint(&mut self, inst: &AgcInst) -> u16;
    fn edrupt(&mut self, inst: &AgcInst) -> u16;
    fn resume(&mut self, inst: &AgcInst) -> u16;
}

impl <'a>AgcInterrupt for AgcCpu<'a> {
    fn inhint(&mut self, _inst: &AgcInst) -> u16 {
        self.gint = false;
        1
    }

    fn relint(&mut self, _inst: &AgcInst) -> u16 {
        self.gint = true;
        1
    }

    fn edrupt(&mut self, _inst: &AgcInst) -> u16 {
        // Inhibits interrupts
        self.gint = false;

        // Loads the Z register into the ZRUPT register
        // TODO

        // TODO: Takes the next instruction from address 0
        //   which sounds like IR = *Addr(0)

        3
    }

    fn resume(&mut self, _inst: &AgcInst) -> u16 {
        let val = self.read(REG_PC_SHADOW) - 1;
        self.write(REG_PC, val);
        self.ir = self.read(REG_IR);
        self.idx_val = 0;

        // Re-enable interrupts
        self.gint = true;
        self.is_irupt = false;

        2
    }
}

#[cfg(test)]
mod interrupt_instr_unittests {
    #[test]
    fn test_hello() {}
}
