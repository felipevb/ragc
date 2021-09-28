use crate::mem::AgcMemType;
use heapless::spsc::Producer;
use log::{error, warn};

const SG_CDUX: usize = 0o32;
const SG_CDUY: usize = 0o33;
const SG_CDUZ: usize = 0o34;
const SG_OPTY: usize = 0o35;
const SG_OPTX: usize = 0o36;
const SG_PIPAX: usize = 0o37;
const SG_PIPAY: usize = 0o40;
const SG_PIPAZ: usize = 0o41;
const _SG_RCHP: usize = 0o42;
const _SG_RCHY: usize = 0o43;
const _SG_RCHR: usize = 0o44;
const SG_INLINK: usize = 0o45;
const _SG_RNRAD: usize = 0o46;
const _SG_GYROCTR: usize = 0o47;
const SG_CDUXCMD: usize = 0o50;
const SG_CDUYCMD: usize = 0o51;
const SG_CDUZCMD: usize = 0o52;
const _SG_OPTYCMD: usize = 0o53;
const _SG_OPTXCMD: usize = 0o54;
const _SG_THRUST: usize = 0o55; // LM only
const _SG_LEMONM: usize = 0o56; // LM only
const SG_OUTLINK: usize = 0o57;
const _SG_ALTM: usize = 0o60; // LM Only

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
