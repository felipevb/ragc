use crate::mem::AgcMemType;
use crate::consts::special::*;
use heapless::spsc::Producer;
use log::{error, warn};

// =============================================================================
// Public Structures
// =============================================================================
#[derive(Clone)]
pub struct AgcSpecialRegs {
    pub cdu: (u16, u16, u16),
    pub opt: (u16, u16),
    pub pipa: (u16, u16, u16),

    // LM only
    pub rch: (u16, u16, u16), // Pitch, Yaw, Roll

    // Uplink from Ground station comes here. Then UPRUPT interrupt
    // occurs. Values can be:
    // - 0 - Error Recovery
    // - cccccCCCCCccccc - Triply Redundant bit pattern
    pub inlink: u16,
}

// =============================================================================
// Implementations
// =============================================================================
impl AgcSpecialRegs {
    pub fn new(_rupt_tx: Producer<u8, 8>) -> Self {
        Self {
            cdu: (0, 0, 0),
            inlink: 0,
            opt: (0, 0),
            pipa: (0, 0, 0),
            rch: (0, 0, 0),
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {}
}

impl AgcMemType for AgcSpecialRegs {
    fn read(&self, bank_idx: usize, bank_offset: usize) -> u16 {
        if bank_idx != 0 {
            error!("Accessing SpecialRegs on a non-zero bank. 0x{:x}", bank_idx);
            return 0;
        }

        match bank_offset {
            SG_CDUX => self.cdu.0,
            SG_CDUY => self.cdu.1,
            SG_CDUZ => self.cdu.2,
            SG_OPTX => self.opt.0,
            SG_OPTY => self.opt.1,
            SG_PIPAX => self.pipa.0,
            SG_PIPAY => self.pipa.1,
            SG_PIPAZ => self.pipa.2,

            // Inlink and Outlink Registers
            SG_INLINK => self.inlink,
            SG_OUTLINK => {
                error!("Reading from outlink, which is known to not be used!");
                0
            }
            SG_CDUXCMD | SG_CDUYCMD | SG_CDUZCMD => 0,
            _ => {
                error!(
                    "Accessing invalid SpecialRegister value: 0o{:o}",
                    bank_offset
                );
                0
            }
        }
    }

    fn write(&mut self, _bank_idx: usize, bank_offset: usize, value: u16) {
        match bank_offset {
            // Block of Read Only Registers. Send a warning mentioning how the
            // Execution is trying to write to special read only registers
            SG_CDUX | SG_CDUY | SG_CDUZ | SG_OPTX | SG_OPTY | SG_PIPAX | SG_PIPAY | SG_PIPAZ => {
                warn!(
                    "Attempting to write to Read-Only Special Registers Address: {:o}",
                    bank_offset
                );
            }

            // Inlink and Outlink Registers
            SG_INLINK => {
                self.inlink = value & 0x7FFF;
            }
            SG_OUTLINK => {
                error!("Writing to outlink, which is known to not be used!");
            }

            _ => {
                error!("Unimplemented Special Write: {:o}", bank_offset);
            }
        }
    }
}
