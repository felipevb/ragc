use super::AgcInst;
use crate::cpu::*;
use crate::utils::{overflow_correction, sign_extend};

use log::debug;

pub trait AgcIo {
    fn ror(&mut self, inst: &AgcInst) -> u16 ;
    fn rand(&mut self, inst: &AgcInst) -> u16 ;
    fn wor(&mut self, inst: &AgcInst) -> u16 ;
    fn wand(&mut self, inst: &AgcInst) -> u16 ;
    fn read_instr(&mut self, inst: &AgcInst) -> u16 ;
    fn write_instr(&mut self, inst: &AgcInst) -> u16 ;
    fn rxor(&mut self, inst: &AgcInst) -> u16 ;
}

impl AgcIo for AgcCpu {
    ///
    /// ## ROR instruction
    ///
    ///  The ROR instruction performs a logical OR between register A and
    ///  I/O address at K. The value is stored in A.
    ///
    /// ### Parameters
    ///
    ///   - inst - `AgcInst` structure that contains the current
    ///     PC data to be used to find the K value for the instruction
    ///
    /// ### Notes
    ///
    /// If the source is the 16-bit Q register, then the full 16-bit value is
    /// logically ORed with A.  Otherwise, the 15-bit source is logically ORed
    /// with the overflow-corrected accumulator, and the result is sign-extended
    /// to 16 bits before storage in A.
    ///
    fn ror(&mut self, inst: &AgcInst) -> u16  {
        let k = inst.get_data_bits() & 0x1FF;
        let io_val = self.read_io(k as usize);

        match k {
            2 => {
                let val = self.read_s16(REG_A) | io_val;
                self.write_s16(REG_A, val);
            }
            _ => {
                let n = self.read_s15(REG_A) | (io_val & 0x7FFF);
                self.write_s15(REG_A, n & 0x7FFF);
            }
        };
        2
    }

    ///
    /// ## RAND instruction
    ///
    ///  The RAND instruction performs a logical AND between register A and
    ///  I/O address at K. The value is stored in A.
    ///
    /// ### Parameters
    ///
    ///   - inst - `AgcInst` structure that contains the current
    ///     PC data to be used to find the K value for the instruction
    ///
    /// ### Notes
    ///
    /// If the source is the 16-bit Q register, then the full 16-bit value is
    /// logically ANDed with A.  Otherwise, the 15-bit source is logically ANDed
    /// with the overflow-corrected accumulator, and the result is sign-extended
    /// to 16 bits before storage in A.
    ///
    fn rand(&mut self, inst: &AgcInst) -> u16  {
        let k = inst.get_data_bits() & 0x1FF;
        let io_val = self.read_io(k as usize);

        match k {
            2 => {
                let val = self.read_s16(REG_A) & io_val;
                self.write_s16(REG_A, val);
            }
            _ => {
                let n = self.read_s15(REG_A) & (io_val & 0x7FFF);
                self.write_s15(REG_A, n & 0x7FFF);
            }
        };
        2
    }

    ///
    /// ## RXOR instruction
    ///
    ///  The RXOR instruction performs a logical XOR between register A and
    ///  I/O address at K. The value is stored in A.
    ///
    /// ### Parameters
    ///
    ///   - inst - `AgcInst` structure that contains the current
    ///     PC data to be used to find the K value for the instruction
    ///
    /// ### Notes
    ///
    /// If the source is the 16-bit Q register, then the full 16-bit value is
    /// logically XORed with A.  Otherwise, the 15-bit source is logically XORed
    /// with the overflow-corrected accumulator, and the result is sign-extended
    /// to 16 bits before storage in A.
    ///
    fn rxor(&mut self, inst: &AgcInst) -> u16  {
        let k = inst.get_data_bits() & 0x1FF;
        let io_val = self.read_io(k as usize);

        match k {
            2 => {
                let val = self.read_s16(REG_A) ^ io_val;
                self.write_s16(REG_A, val);
            }
            _ => {
                let n = self.read_s15(REG_A) ^ (io_val & 0x7FFF);
                self.write_s15(REG_A, n & 0x7FFF);
            }
        };
        2
    }

    ///
    /// ## WOR instruction
    ///
    ///  The WOR instruction performs a logical OR between register A and
    ///  I/O address at K. The value is stored in both A and in I/O address K.
    ///
    /// ### Parameters
    ///
    ///   - inst - `AgcInst` structure that contains the current
    ///     PC data to be used to find the K value for the instruction
    ///
    /// ### Notes
    ///
    /// If the destination is the 16-bit Q register, then the full 16-bit value is
    /// logically ORed with A and stored at both A and K.  Otherwise, the 15-bit
    /// destination is logically ORed with the overflow-corrected accumulator and
    /// stored to K, and the result is sign-extended to 16 bits before storage in A.
    ///
    fn wor(&mut self, inst: &AgcInst) -> u16  {
        let k: usize = (inst.get_data_bits() & 0x1FF) as usize;
        let io_val = self.read_io(k);

        match k {
            2 => {
                let n = self.read_s16(REG_A) | io_val;
                debug!("WOR: {:06o} | {:06o} => {:06o}", k, io_val, n);
                self.write_s16(REG_A, n);
                self.write_io(k, n);
            }
            _ => {
                let n = self.read_s15(REG_A) | (io_val & 0x7FFF);
                debug!("WOR: {:06o} | {:06o} => {:06o}", k, io_val, n);
                self.write_s15(REG_A, n);
                self.write_io(k, n & 0x7FFF);
            }
        };
        2
    }

    ///
    /// ## WAND instruction
    ///
    ///  The WAND instruction performs a logical AND between register A and
    ///  I/O address at K. The value is stored in both A and in I/O address K.
    ///
    /// ### Parameters
    ///
    ///   - inst - `AgcInst` structure that contains the current
    ///     PC data to be used to find the K value for the instruction
    ///
    /// ### Notes
    ///
    /// If the destination is the 16-bit Q register, then the full 16-bit value is
    /// logically ANDed with A and stored at both A and K.  Otherwise, the 15-bit
    /// destination is logically ANDed with the overflow-corrected accumulator and
    /// stored to K, and the result is sign-extended to 16 bits before storage in A.
    ///
    fn wand(&mut self, inst: &AgcInst) -> u16  {
        let k: usize = (inst.get_data_bits() & 0x1FF) as usize;
        let io_val = self.read_io(k);

        match k {
            2 => {
                let n = self.read_s16(REG_A) & io_val;
                self.write_s16(REG_A, n);
                self.write_io(k, n);
            }
            _ => {
                let n = self.read_s15(REG_A) & (io_val & 0x7FFF);
                self.write_s15(REG_A, n);
                self.write_io(k, n & 0x7FFF);
            }
        };
        2
    }

    ///
    /// ## READ instruction
    ///
    /// ### Parameters
    ///   - inst - `AgcInst` structure that contains the current
    ///     PC data to be used to find the K value for the instruction
    ///
    /// ### Notes
    ///   - If Reading from Q register (IO space mapped to Q register (2)), then
    ///     the 16-bit value is read. Otherwise, value to sign extended and stored
    ///     into A register
    ///
    fn read_instr(&mut self, inst: &AgcInst) -> u16  {
        let k = inst.get_data_bits() & 0x1FF;
        let io_val = match k {
            2 => self.read_io(k as usize),
            _ => sign_extend(self.read_io(k as usize)),
        };
        self.write_s16(REG_A, io_val);
        2
    }

    ///
    /// ## Write instruction
    ///
    /// ### Parameters
    ///   - inst - `AgcInst` structure that contains the current
    ///     PC data to be used to find the K value for the instruction
    ///
    /// ### Notes
    ///   - If the destination is the 16-bit Q register, then the full
    ///     16-bit value of A is stored into K.  Otherwise, the value is
    ///     overflow-corrected before storage.
    ///
    fn write_instr(&mut self, inst: &AgcInst) -> u16  {


        let k = inst.get_data_bits() & 0x1FF;
        let val = self.read_s16(REG_A);
        match k {
            2 => {
                self.write_io(k as usize, val);
            }
            _ => {
                self.write_io(k as usize, overflow_correction(val) & 0x7FFF);
            }
        }
        2
    }
}

#[cfg(test)]
mod io_tests {
    use crate::cpu;
    use crate::instr::tests::{init_agc, validate_cpu_state};

    ///
    /// ## READ instruction test
    ///
    ///   This test performs basic READs from the mapped `REG_L` and `REG_Q`
    ///   that can be easily controlled via the unit testing. Using the two IO
    ///   address that are mapped to these registers, we test the 15-bit and
    ///   16-bit operations being performed.
    ///
    #[test]
    fn io_read_test() {
        let mut cpu = init_agc();
        let src = [
            (cpu::REG_L, 0x7FDD, 1, 0xFFDD),
            (cpu::REG_L, 0x3FCC, 1, 0x3FCC),
            (cpu::REG_Q, 0x7FAA, 2, 0x7FAA),
            (cpu::REG_Q, 0xFFBB, 2, 0xFFBB),
        ];

        for (reg, reg_val, idx, expect_result) in src.iter() {
            let inst_data: u16 = 0o00000 + *idx as u16;

            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);
            cpu.reset();

            cpu.write(*reg, *reg_val);
            cpu.step();
            cpu.step();

            validate_cpu_state(&mut cpu, 0x802);
            assert_eq!(cpu.read(cpu::REG_A), *expect_result);
        }
    }

    ///
    /// ## ROR instruction test
    ///
    ///   This test performs basic WORs to the mapped `REG_L` and `REG_Q`
    ///   that can be easily controlled via the unit testing. Using the two IO
    ///   address that are mapped to these registers, we test the 15-bit and
    ///   16-bit operations being performed.
    ///
    #[test]
    fn io_ror_test() {
        let mut cpu = init_agc();
        let src = [
            // Try basic ORs that don't deal with signed / etc
            (0x00AA, cpu::REG_L, 0x0055, 1, 0x00FF),
            (0x00AA, cpu::REG_Q, 0x0055, 2, 0x00FF),
            // Test sign extension for A
            (0x00AA, cpu::REG_L, 0x7000, 1, 0xF0AA),
            (0x00AA, cpu::REG_Q, 0xF000, 2, 0xF0AA),
            // Overflow Corrected testing
            (0x40AA, cpu::REG_L, 0x0100, 1, 0x01AA),
            (0x80AA, cpu::REG_L, 0x0100, 1, 0xC1AA),
            // Overflow and Signed Testing
            (0x40AA, cpu::REG_L, 0x4F00, 1, 0xCFAA),
            // 16 Bit OR
            (0xAA0A, cpu::REG_Q, 0x5550, 2, 0xFF5A),
            (0x0A0A, cpu::REG_Q, 0x5550, 2, 0x5F5A),
        ];

        for (a_val, reg, reg_val, idx, expect_a_result) in src.iter() {
            let inst_data: u16 = 0o04000 + *idx as u16;

            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);
            cpu.reset();

            cpu.write(cpu::REG_A, *a_val);
            cpu.write(*reg, *reg_val);
            cpu.step();
            cpu.step();

            validate_cpu_state(&mut cpu, 0x802);
            assert_eq!(cpu.read(cpu::REG_A), *expect_a_result);
        }
    }

    ///
    /// ## RAND instruction test
    ///
    ///   This test performs basic RANDs to the mapped `REG_L` and `REG_Q`
    ///   that can be easily controlled via the unit testing. Using the two IO
    ///   address that are mapped to these registers, we test the 15-bit and
    ///   16-bit operations being performed.
    ///
    #[test]
    fn io_rand_test() {
        let mut cpu = init_agc();
        let src = [
            // Try basic ORs that don't deal with signed / etc
            (0x00AA, cpu::REG_L, 0x00FF, 1, 0x00AA),
            (0x00AA, cpu::REG_Q, 0x00FF, 2, 0x00AA),
            // Test sign extension for A
            (0xC0AA, cpu::REG_L, 0x70FF, 1, 0xC0AA),
            (0xC0AA, cpu::REG_Q, 0xF0FF, 2, 0xC0AA),
            // Overflow Corrected testing
            (0x40AA, cpu::REG_L, 0x4000, 1, 0x0000),
            (0x80AA, cpu::REG_L, 0x4000, 1, 0xC000),
            (0x40AA, cpu::REG_Q, 0x4000, 2, 0x4000),
            (0x80AA, cpu::REG_Q, 0x4000, 2, 0x0000),
            // Overflow and Signed Testing
            (0x80AA, cpu::REG_L, 0x40FF, 1, 0xC0AA),
            // 16 Bit AND
            (0xAAAA, cpu::REG_Q, 0xAAA0, 2, 0xAAA0),
            (0x5555, cpu::REG_Q, 0x5550, 2, 0x5550),
        ];

        for (a_val, reg, reg_val, idx, expect_a_result) in src.iter() {
            let inst_data: u16 = 0o02000 + *idx as u16;

            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);
            cpu.reset();

            cpu.write(cpu::REG_A, *a_val);
            cpu.write(*reg, *reg_val);
            cpu.step();
            cpu.step();

            validate_cpu_state(&mut cpu, 0x802);
            assert_eq!(cpu.read(cpu::REG_A), *expect_a_result);
        }
    }

    ///
    /// ## RXOR instruction test
    ///
    ///   This test performs basic RXORs to the mapped `REG_L` and `REG_Q`
    ///   that can be easily controlled via the unit testing. Using the two IO
    ///   address that are mapped to these registers, we test the 15-bit and
    ///   16-bit operations being performed.
    ///
    #[test]
    fn io_rxor_test() {
        let mut cpu = init_agc();
        let src = [
            // Try basic ORs that don't deal with signed / etc
            (0x00AA, cpu::REG_L, 0x0055, 1, 0x00FF),
            (0x00AA, cpu::REG_Q, 0x0055, 2, 0x00FF),
            // Test sign extension for A
            (0x00AA, cpu::REG_L, 0x4000, 1, 0xC0AA),
            (0x80AA, cpu::REG_Q, 0x4000, 2, 0xC0AA),
            //// Overflow Corrected testing
            (0x40AA, cpu::REG_L, 0x0000, 1, 0x00AA),
            (0x80AA, cpu::REG_L, 0x0000, 1, 0xC0AA),
            // Overflow and Signed Testing
            (0x40AA, cpu::REG_L, 0x4000, 1, 0xC0AA),
            // 16 Bit XOR
            (0xAAAA, cpu::REG_Q, 0x5550, 2, 0xFFFA),
            (0x5555, cpu::REG_Q, 0x5550, 2, 0x0005),
        ];

        for (a_val, reg, reg_val, idx, expect_a_result) in src.iter() {
            let inst_data: u16 = 0o06000 + *idx as u16;

            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);
            cpu.reset();

            cpu.write(cpu::REG_A, *a_val);
            cpu.write(*reg, *reg_val);
            cpu.step();
            cpu.step();

            validate_cpu_state(&mut cpu, 0x802);
            assert_eq!(cpu.read(cpu::REG_A), *expect_a_result);
        }
    }

    ///
    /// ## WRITE instruction test
    ///
    ///   This test performs basic WRITEs to the mapped `REG_L` and `REG_Q`
    ///   that can be easily controlled via the unit testing. Using the two IO
    ///   address that are mapped to these registers, we test the 15-bit and
    ///   16-bit operations being performed.
    ///
    #[test]
    fn io_write_test() {
        let mut cpu = init_agc();
        let src = [
            (cpu::REG_L, 0xFFDD, 1, 0x7FDD),
            (cpu::REG_L, 0x3FCC, 1, 0x3FCC),
            (cpu::REG_Q, 0x7FAA, 2, 0x7FAA),
            (cpu::REG_Q, 0xFFBB, 2, 0xFFBB),
        ];

        for (reg, reg_val, idx, expect_result) in src.iter() {
            let inst_data: u16 = 0o01000 + *idx as u16;

            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);
            cpu.reset();

            cpu.write(cpu::REG_A, *reg_val);
            cpu.step();
            cpu.step();

            validate_cpu_state(&mut cpu, 0x802);
            assert_eq!(cpu.read(*reg), *expect_result);
        }
    }

    ///
    /// ## WOR instruction test
    ///
    ///   This test performs basic WORs to the mapped `REG_L` and `REG_Q`
    ///   that can be easily controlled via the unit testing. Using the two IO
    ///   address that are mapped to these registers, we test the 15-bit and
    ///   16-bit operations being performed.
    ///
    #[test]
    fn io_wor_test() {
        let mut cpu = init_agc();
        let src = [
            // Try basic ORs that don't deal with signed / etc
            (0x00AA, cpu::REG_L, 0x0055, 1, 0x00FF, 0x00FF),
            (0x00AA, cpu::REG_Q, 0x0055, 2, 0x00FF, 0x00FF),
            // Test sign extension for A
            (0x00AA, cpu::REG_L, 0x7000, 1, 0xF0AA, 0x70AA),
            (0x00AA, cpu::REG_Q, 0xF000, 2, 0xF0AA, 0xF0AA),
            // Overflow Corrected testing
            (0x40AA, cpu::REG_L, 0x0100, 1, 0x01AA, 0x01AA),
            (0x80AA, cpu::REG_L, 0x0100, 1, 0xC1AA, 0x41AA),
            // Overflow and Signed Testing
            (0x40AA, cpu::REG_L, 0x4F00, 1, 0xCFAA, 0x4FAA),
            // 16 Bit OR
            (0xAA0A, cpu::REG_Q, 0x5550, 2, 0xFF5A, 0xFF5A),
            (0x0A0A, cpu::REG_Q, 0x5550, 2, 0x5F5A, 0x5F5A),
        ];

        for (a_val, reg, reg_val, idx, expect_a_result, expect_reg_result) in src.iter() {
            let inst_data: u16 = 0o05000 + *idx as u16;

            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);
            cpu.reset();

            cpu.write(cpu::REG_A, *a_val);
            cpu.write(*reg, *reg_val);
            cpu.step();
            cpu.step();

            validate_cpu_state(&mut cpu, 0x802);
            assert_eq!(cpu.read(cpu::REG_A), *expect_a_result);
            assert_eq!(cpu.read(*reg), *expect_reg_result);
        }
    }

    ///
    /// ## WAND instruction test
    ///
    ///   This test performs basic WANDs to the mapped `REG_L` and `REG_Q`
    ///   that can be easily controlled via the unit testing. Using the two IO
    ///   address that are mapped to these registers, we test the 15-bit and
    ///   16-bit operations being performed.
    ///
    #[test]
    fn io_wand_test() {
        let mut cpu = init_agc();
        let src = [
            // Try basic ORs that don't deal with signed / etc
            (0x00AA, cpu::REG_L, 0x00FF, 1, 0x00AA, 0x00AA),
            (0x00AA, cpu::REG_Q, 0x00FF, 2, 0x00AA, 0x00AA),
            // Test sign extension for A
            (0xC0AA, cpu::REG_L, 0x70FF, 1, 0xC0AA, 0x40AA),
            (0xC0AA, cpu::REG_Q, 0xF0FF, 2, 0xC0AA, 0xC0AA),
            // Overflow Corrected testing
            (0x40AA, cpu::REG_L, 0x4000, 1, 0x0000, 0x0000),
            (0x80AA, cpu::REG_L, 0x4000, 1, 0xC000, 0x4000),
            (0x40AA, cpu::REG_Q, 0x4000, 2, 0x4000, 0x4000),
            (0x80AA, cpu::REG_Q, 0x4000, 2, 0x0000, 0x0000),
            // Overflow and Signed Testing
            (0x80AA, cpu::REG_L, 0x40FF, 1, 0xC0AA, 0x40AA),
            // 16 Bit AND
            (0xAAAA, cpu::REG_Q, 0xAAA0, 2, 0xAAA0, 0xAAA0),
            (0x5555, cpu::REG_Q, 0x5550, 2, 0x5550, 0x5550),
        ];

        for (a_val, reg, reg_val, idx, expect_a_result, expect_reg_result) in src.iter() {
            let inst_data: u16 = 0o03000 + *idx as u16;

            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);
            cpu.reset();

            cpu.write(cpu::REG_A, *a_val);
            cpu.write(*reg, *reg_val);
            cpu.step();
            cpu.step();

            validate_cpu_state(&mut cpu, 0x802);
            assert_eq!(cpu.read(cpu::REG_A), *expect_a_result);
            assert_eq!(cpu.read(*reg), *expect_reg_result);
        }
    }
}
