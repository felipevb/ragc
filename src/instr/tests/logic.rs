#[cfg(test)]
mod logic_tests {
    use crate::cpu;
    use crate::instr::tests::{init_agc, validate_cpu_state};

    ///
    /// ## MASK instruction test
    ///
    ///   This test performs MASK (or AND) instruction. This will test 15bit,
    ///   16bits, overflow correction, and sign extension
    ///
    #[test]
    fn logic_mask_test() {
        let mut cpu = init_agc();
        let src = [
            // Try basic ORs that don't deal with signed / etc
            (0x00AA, 0x200, 0x00FF, 0x00AA),
            (0x00AA, cpu::REG_Q, 0x00FF, 0x00AA),
            // Test sign extension for A
            (0xC0AA, 0x200, 0x70FF, 0xC0AA),
            (0xC0AA, cpu::REG_Q, 0xF0FF, 0xC0AA),
            // Overflow Corrected testing
            (0x40AA, 0x100, 0x7FFF, 0x00AA),
            (0x80AA, 0x100, 0x40FF, 0xC0AA),
            (0x40AA, cpu::REG_Q, 0x4000, 0x4000),
            (0x80AA, cpu::REG_Q, 0x4000, 0x0000),
            // Overflow and Signed Testing
            (0x80AA, 0x100, 0x40FF, 0xC0AA),
            // 16 Bit AND
            (0xAAAA, cpu::REG_Q, 0xAAA0, 0xAAA0),
            (0x5555, cpu::REG_Q, 0x5550, 0x5550),
        ];

        for (a_val, idx, reg_val, expect_a_result) in src.iter() {
            let inst_data: u16 = 0o70000 + *idx as u16;

            cpu.write(0x800, inst_data);
            cpu.reset();

            cpu.write(cpu::REG_A, *a_val);
            cpu.write(*idx, *reg_val);
            cpu.step();

            validate_cpu_state(&cpu, 0x801);
            assert_eq!(cpu.read(cpu::REG_A), *expect_a_result);
        }
    }
}
