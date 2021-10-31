pub mod arith;
pub mod cf;
pub mod intrpt;
pub mod io;
pub mod ldst;
pub mod logic;

pub use arith::AgcArith;
pub use cf::AgcControlFlow;
pub use intrpt::AgcInterrupt;
pub use io::AgcIo;
pub use ldst::AgcLoadStore;
pub use logic::AgcLogic;

pub mod tests;

const DATA_MASK: u16 = 0o7777; // 0xFFF
const DATA_MASK_RAM: u16 = 0o1777; // 0x3FF
const OPCODE_MASK: u16 = 0o7;
const OPCODE_OFFSET: u16 = 12;
const OPCODE_EXTEND_MASK: u16 = 0o100000; // 0x8000;

#[derive(Debug)]
pub enum AgcMnem {
    AD,
    ADS,
    AUG,
    BZF,
    BZMF,
    CA,
    CS,
    CCS,
    DAS,
    DCA,
    DCS,
    DIM,
    DV,
    DXCH,
    EDRUPT,
    EXTEND,
    INCR,
    INDEX,
    INHINT,
    LXCH,
    MASK,
    MP,
    MSU,
    QXCH,
    RAND,
    READ,
    RELINT,
    RESUME,
    ROR,
    RXOR,
    SU,
    TC,
    TCF,
    TS,
    WAND,
    WOR,
    WRITE,
    XCH,
    INVALID,
}

#[derive(Debug)]
pub struct AgcInst {
    pub pc: u16,
    pub mnem: AgcMnem,
    pub inst_data: u16,
    pub extrabits: Option<u8>,
    pub mct: u8,
}

impl AgcInst {
    #[allow(dead_code)]
    pub fn new() -> AgcInst {
        AgcInst {
            pc: 0o00000,
            inst_data: 0o00000,
            mnem: AgcMnem::INVALID,
            extrabits: None,
            mct: 1,
        }
    }

    pub fn get_opcode_bits(&self) -> u8 {
        ((self.inst_data >> OPCODE_OFFSET) & OPCODE_MASK) as u8
    }

    pub fn get_data_bits(&self) -> u16 {
        (self.inst_data & DATA_MASK) as u16
    }

    pub fn get_kaddr(&self) -> usize {
        (self.inst_data & DATA_MASK) as usize
    }

    ///
    /// Function to quickly get the K value that is specifically only the
    /// addressable RAM address. This is used for instructions that used more
    /// than 3 bits for the opcode.
    ///
    /// # Result
    ///
    ///  - usize - Address that is within the RAM addressable range.
    ///
    pub fn get_kaddr_ram(&self) -> usize {
        let v = (self.inst_data & DATA_MASK_RAM) as usize;
        v
    }

    ///
    /// Function getter to get whether the current instruction is an extended
    ///   instruction or a basic instruction.
    ///
    /// # Result
    ///
    ///  - bool - True if the instruction is an extended instruction, false if
    ///           the instruction is a basic instruction.
    ///
    pub fn is_extended(&self) -> bool {
        let val = self.inst_data & OPCODE_EXTEND_MASK;
        if val == OPCODE_EXTEND_MASK {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod disasm_tests {
    use super::AgcInst;
    use super::AgcMnem;
    use super::DATA_MASK_RAM;

    #[test]
    ///
    /// Function is to test to check the get_kaddr_ram functionality to see it
    /// is properly returning the correct RAM address.
    ///
    fn test_instr_get_kaddr_ram() {
        // Check the condition where all values should be false. All values within
        // this test has BIT15 set to 0.
        for test_val in 0o00000..=0o177777 {
            let instr = AgcInst {
                pc: 0o00000,
                inst_data: test_val,
                mnem: AgcMnem::INVALID,
                extrabits: None,
                mct: 1,
            };

            let kaddr = (DATA_MASK_RAM as usize) & (test_val as usize);
            assert_eq!(
                kaddr,
                instr.get_kaddr_ram(),
                "Test failed for `get_kaddr_ram()`. Expected: {:?} | Value: {:?}",
                kaddr,
                instr.get_kaddr_ram()
            );
        }
    }

    #[test]
    ///
    /// Function is to test to check the is_extended functionality to see it
    /// is properly returning the correct value for all values possible.
    ///
    fn test_instr_extend() {
        // Check the condition where all values should be false. All values within
        // this test has BIT15 set to 0.
        for test_val in 0o00000..=0o77777 {
            let instr = AgcInst {
                pc: 0o00000,
                inst_data: test_val,
                mnem: AgcMnem::INVALID,
                extrabits: None,
                mct: 1,
            };
            assert_eq!(
                false,
                instr.is_extended(),
                "Test failed for `is_extended`. Expected: {:?} | Value: {:?}",
                false,
                instr.is_extended()
            );
        }
        // Check the condition where all values should be True. All values within
        // this test has BIT15 set to 1.
        for test_val in 0o100000..=0o177777 {
            let instr = AgcInst {
                pc: 0o00000,
                inst_data: test_val,
                mnem: AgcMnem::INVALID,
                extrabits: None,
                mct: 1,
            };
            assert_eq!(
                true,
                instr.is_extended(),
                "Test failed for `is_extended`. Expected: {:?} | Value: {:?}",
                true,
                instr.is_extended()
            );
        }
    }
}
