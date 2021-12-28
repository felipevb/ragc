use crate::mem::AgcMemType;
use crate::consts::edit::{*};
use log::{error, trace};

#[derive(Clone)]
pub struct AgcEditRegs {
    cyr: u16,
    sr: u16,
    cyl: u16,
    edop: u16,
}

impl AgcEditRegs {
    pub fn new() -> Self {
        Self {
            cyl: 0,
            cyr: 0,
            sr: 0,
            edop: 0,
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.cyl = 0;
        self.cyr = 0;
        self.sr = 0;
        self.edop = 0;
    }
}

impl AgcMemType for AgcEditRegs {
    fn read(&self, _bank_idx: usize, bank_offset: usize) -> u16 {
        trace!("Edit Read: 0o{:o}", bank_offset);
        match bank_offset {
            SG_CYL => self.cyl,
            SG_CYR => self.cyr,
            SG_SR => self.sr,
            SG_EDOP => self.edop,
            _ => {
                error!("Invalid EditRegister Read: 0o{:o}", bank_offset);
                0
            }
        }
    }

    fn write(&mut self, _bank_idx: usize, bank_offset: usize, value: u16) {
        let newval = value & 0x7FFF;
        trace!("Edit Write: 0o{:o}", bank_offset);

        match bank_offset {
            SG_CYL => {
                let bitval = newval & 0x4000;
                self.cyl = (newval << 1) & 0x7FFF;
                self.cyl |= bitval >> 14;
            }
            SG_CYR => {
                let bitval = newval & 0x1;
                self.cyr = (newval >> 1) | (bitval << 14)
            }
            SG_SR => {
                let bitval = newval & 0o40000;
                self.sr = (newval >> 1) | bitval;
            }
            SG_EDOP => self.edop = (newval >> 7) & 0o177,
            _ => {
                error!("Invalid EditRegister Write: {:o}", bank_offset);
            }
        }
    }
}

#[cfg(test)]
mod utils_tests {
    use super::*;

    #[test]
    fn sr_tests() {
        let test_vals = [
            // Test the zero case
            (0o77777, 0o77777),
            (0, 0),
            // Test basic values on their shifting (0 bit)
            (0o1, 0o0),
            (0o07777, 0o03777),
            (0o02525, 0o01252),
            (0o40001, 0o60000),
            (0o47777, 0o63777),
            (0o42525, 0o61252),
        ];

        for (input, output) in test_vals.iter() {
            let mut edit = AgcEditRegs::new();

            edit.write(0, SG_SR, *input);
            let res = edit.read(0, SG_SR);
            assert_eq!(res, *output, "Failed SR testing: {:o} | {:o}", *output, res);
        }
    }

    #[test]
    fn edop_tests() {
        let test_vals = [
            // Test the zero case
            (0o77777, 0o00177),
            (0, 0),
            // Test basic values on their shifting (0 bit)
            (0o1, 0o0),
            (0o07777, 0o00037),
            (0o02525, 0o00012),
            (0o40000, 0o00000),
        ];

        for (input, output) in test_vals.iter() {
            let mut edit = AgcEditRegs::new();

            edit.write(0, SG_EDOP, *input);
            let res = edit.read(0, SG_EDOP);
            assert_eq!(
                res, *output,
                "Failed EDOP testing: {:o} | {:o}",
                *output, res
            );
        }
    }

    #[test]
    fn cyr_tests() {
        let test_vals = [
            // Test the zero case
            (0o77777, 0o77777),
            (0, 0),
            // Test basic values on their shifting (0 bit)
            (0o1, 0o40000),
            (0o40001, 0o60000),
            (0o40000, 0o20000),
        ];

        for (input, output) in test_vals.iter() {
            let mut edit = AgcEditRegs::new();

            edit.write(0, SG_CYR, *input);
            let res = edit.read(0, SG_CYR);
            assert_eq!(
                res, *output,
                "Failed CYR testing: {:o} | {:o}",
                *output, res
            );
        }
    }

    #[test]
    fn cyl_tests() {
        let test_vals = [
            // Test the zero case
            (0o77777, 0o77777),
            (0, 0),
            // Test basic values on their shifting (0 bit)
            (0o00001, 0o00002),
            (0o40001, 0o00003),
            (0o60000, 0o40001),
        ];

        for (input, output) in test_vals.iter() {
            let mut edit = AgcEditRegs::new();

            edit.write(0, SG_CYL, *input);
            let res = edit.read(0, SG_CYL);
            assert_eq!(
                res, *output,
                "Failed CYL testing: {:o} | {:o}",
                *output, res
            );
        }
    }
}
