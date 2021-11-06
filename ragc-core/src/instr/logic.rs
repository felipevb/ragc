use super::AgcInst;
use crate::cpu;
use crate::cpu::*;

pub trait AgcLogic {
    fn mask(&mut self, inst: &AgcInst) -> u16;
}

impl <'a>AgcLogic for AgcCpu<'a> {
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
    fn mask(&mut self, inst: &AgcInst) -> u16 {
        let k = inst.get_kaddr();
        match k {
            cpu::REG_A | cpu::REG_Q => {
                let mut val = self.read_s16(k);
                val = self.read_s16(REG_A) & val;
                self.write_s16(REG_A, val);
            }
            _ => {
                let val = self.read_s15(k);
                let a = self.read_s15(REG_A);
                let n = a & (val & 0x7FFF);
                self.write_s15(REG_A, n & 0x7FFF);
            }
        };
        2
    }
}
