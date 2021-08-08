use super::AgcInst;
use crate::cpu;
use crate::cpu::*;
use log::trace;

pub trait AgcLogic {
    fn mask(&mut self, inst: &AgcInst) -> bool;
}

impl AgcLogic for AgcCpu {
    ///
    /// ## MASK instruction
    ///
    ///  The MASK instruction performs a logical AND between register A and
    ///  Memory address at K. The value is stored in A.
    ///
    /// ### Parameters
    ///
    ///   - inst - `AgcInst` structure that contains the current
    ///     PC data to be used to find the K value for the instruction
    ///
    /// ### Notes
    ///
    /// If the source is 16-bit, then the full 16-bit value is logically ANDed
    /// with A.  Otherwise, the 15-bit source is logically ANDed with the
    /// overflow-corrected accumulator, and the result is sign-extended to
    /// 16-bits before storage in A.
    ///
    fn mask(&mut self, inst: &AgcInst) -> bool {
        self.cycles = 2;
        let k = inst.get_kaddr();
        match k {
            cpu::REG_A | cpu::REG_Q => {
                let val = self.read_s16(k);
                trace!("MASK (S16) {:x}", val);
                self.write_s16(REG_A, self.read_s16(REG_A) & val);
            }
            _ => {
                let val = self.read_s15(k);
                trace!("MASK (S15) {:x}", val);
                let a = self.read_s15(REG_A);
                let n = a & (val & 0x7FFF);
                self.write_s15(REG_A, n & 0x7FFF);
            }
        };
        true
    }
}
