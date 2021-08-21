#[cfg(test)]
mod arith_tests {
    use crate::cpu;
    use crate::instr::tests::{init_agc, validate_cpu_state};

    #[test]
    fn aug_positive() {
        let mut cpu = init_agc();
        let srcs = [(cpu::REG_A, 0x11), (cpu::REG_L, 0x11), (0o200, 0x11)];

        for (i, val) in srcs.iter() {
            let idx = *i;

            let inst_data: u16 = 0o24000 + idx as u16;
            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);
            cpu.reset();

            cpu.write(idx, 0x10);
            cpu.step();
            cpu.step();

            validate_cpu_state(&mut cpu, 0x802);
            assert_eq!(cpu.read_s16(idx), *val);
        }
    }

    #[test]
    fn aug_negative() {
        let mut cpu = init_agc();
        let srcs = [
            (cpu::REG_A, 0xFFFB, 0xFFFA),
            (cpu::REG_Q, 0xFFFB, 0xFFFA),
            (cpu::REG_L, 0x7FFB, 0x7FFA),
            (0o200, 0x7FFB, 0x7FFA),
        ];

        for (i, init, val) in srcs.iter() {
            let idx = *i;

            let inst_data: u16 = 0o24000 + idx as u16;
            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);
            cpu.reset();

            cpu.write(idx, *init);
            cpu.step();
            cpu.step();

            validate_cpu_state(&mut cpu, 0x802);
            assert_eq!(
                cpu.read(idx),
                *val,
                "Failed comparison: e:{:o} | r:{:o}",
                *val,
                cpu.read(idx)
            );
        }
    }

    #[test]
    fn aug_positive_16bit() {
        let mut cpu = init_agc();
        let srcs = [(cpu::REG_A, 0x4000), (cpu::REG_Q, 0x4000)];

        for (i, val) in srcs.iter() {
            let idx = *i;
            let inst_data: u16 = 0o24000 + idx as u16;

            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);
            cpu.reset();

            cpu.write(idx, 0x3FFF); //0o37777
            cpu.step();
            cpu.step();

            validate_cpu_state(&mut cpu, 0x802);
            println!("{:x} | {:x}", cpu.read(idx), *val);
            assert_eq!(
                cpu.read(idx),
                *val,
                "Aug Instruction failed. e{:o} | r:{:o}",
                *val,
                cpu.read(idx)
            );
        }
    }

    // =========================================================================
    // INCR Tests
    // =========================================================================
    #[test]
    fn incr_tests() {
        let mut cpu = init_agc();
        let srcs = [
            (cpu::REG_A, 0x000F, 0x0010),
            (cpu::REG_Q, 0x000F, 0x0010),
            (cpu::REG_L, 0x000F, 0x0010),
            (0o200, 0x000F, 0x0010),
            (cpu::REG_A, 0xFF00, 0xFF01),
            (cpu::REG_Q, 0xFF00, 0xFF01),
            (cpu::REG_L, 0x7F00, 0x7F01),
            (0o200, 0x7F00, 0x7F01),
            (cpu::REG_A, 0xFFFF, 0x0001),
            (cpu::REG_Q, 0xFFFF, 0x0001),
            (cpu::REG_L, 0x7FFF, 0x0001),
            (0o200, 0x7FFF, 0x0001),
            (cpu::REG_A, 0x0000, 0x0001),
            (cpu::REG_Q, 0x0000, 0x0001),
            (cpu::REG_L, 0x0000, 0x0001),
            (0o200, 0x0000, 0x0001),
            // Based on ValidateINCR.agc, Test 12, for 15-bit values
            // need to overflow to 0o0 from 0o37777. Curently I cannot
            // find any documentation of this other than these tests.
            (cpu::REG_Q, 0x0000, 0x0001),
            (cpu::REG_L, 0o37777, 0o00000),
            (0o200, 0o37777, 0o00000),
        ];

        for (i, start_val, expect_res) in srcs.iter() {
            let idx = *i;
            let inst_data: u16 = 0o24000 + idx as u16;

            println!("Testing INCR: {:x} {:x} {:x}", idx, start_val, expect_res);
            cpu.write(0x800, inst_data);
            cpu.reset();

            cpu.write(idx, *start_val);
            cpu.step();

            validate_cpu_state(&mut cpu, 0x801);
            assert_eq!(cpu.read(idx), *expect_res);
        }
    }

    // =========================================================================
    // DV instruction tests
    // =========================================================================
    #[test]
    fn arith_dv_tests_known_set() {
        let mut cpu = init_agc();
        let srcs = [
            // Based on https://www.ibiblio.org/apollo/assembly_language_manual.html,
            // there is a known example table from documented literature from Smally
            // In the form of (REGA, REGL, Kval, REGA(q), REGL(remainder))
            (0o17777, 0o40000, 0o20000, 0o37774, 0o00001),
            //(0o17777, 0o40000, 0o57777, 0o40003, 0o00001),
            (0o60000, 0o37777, 0o20000, 0o40003, 0o77776),
            //(0o60000, 0o37777, 0o57777, 0o37774, 0o77776),
            //(0o17777, 0o37777, 0o20000, 0o37777, 0o17777),
            //(0o37776, 0o00000, 0o37776, 0o37777, 0o37776),
            //(0o00000, 0o77777, 0o00000, 0o40000, 0o77777),
            //(0o00000, 0o77777, 0o77777, 0o37777, 0o77777),
            //(0o77777, 0o00000, 0o00000, 0o37777, 0o00000),
            //(0o77777, 0o00000, 0o77777, 0o40000, 0o00000)
        ];

        for (rega, regl, divisor, quotent, remainder) in srcs.iter() {
            let inst_data: u16 = 0o10200;

            println!(
                "Testing DV: {:o} {:o} {:o} => q: {:o} r: {:o}",
                rega, regl, divisor, quotent, remainder
            );
            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);

            cpu.reset();
            cpu.write_s15(cpu::REG_A, *rega);
            cpu.write_s15(cpu::REG_L, *regl);
            cpu.write_s15(0o200, *divisor);

            cpu.step();
            validate_cpu_state(&mut cpu, 0x801);
            cpu.step();
            validate_cpu_state(&mut cpu, 0x802);

            assert_eq!(cpu.read_s15(cpu::REG_A), *quotent);
            assert_eq!(cpu.read_s15(cpu::REG_L), *remainder);
        }
    }

    #[test]
    fn arith_dim_tests() {
        let mut cpu = init_agc();
        let srcs = [
            (cpu::REG_A, 0o00000, 0o00000),
            (cpu::REG_A, 0o00001, 0o00000),
            (cpu::REG_A, 0o177777, 0o177777),
            (cpu::REG_A, 0o177776, 0o177777),
            (cpu::REG_L, 0o00000, 0o00000),
            (cpu::REG_L, 0o00001, 0o00000),
            (cpu::REG_L, 0o177777, 0o77777),
            (cpu::REG_L, 0o177776, 0o77777),
            (cpu::REG_Q, 0o00000, 0o00000),
            (cpu::REG_Q, 0o00001, 0o00000),
            (cpu::REG_Q, 0o177777, 0o177777),
            (cpu::REG_Q, 0o177776, 0o177777),
            (0o200, 0o00000, 0o00000),
            (0o200, 0o00001, 0o00000),
            (0o200, 0o177777, 0o77777),
            (0o200, 0o177776, 0o77777),
        ];

        for (idx, val, expected) in srcs.iter() {
            let inst_data: u16 = 0o26000 + (*idx as u16);
            //println!("Testing DIM: {:o}: {:o} => {:o} ", *idx, *val, *expected);

            // Write the Instructions and reset the CPU to start at that location
            // with the proper loading of the IR register
            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);
            cpu.reset();

            // Set the Values to Test
            cpu.write(*idx, *val);
            cpu.step();
            validate_cpu_state(&mut cpu, 0x801);
            cpu.step();
            validate_cpu_state(&mut cpu, 0x802);
            assert_eq!(cpu.read(*idx), *expected);
        }
    }

    // =========================================================================
    // MSU instruction tests
    // =========================================================================
    #[test]
    fn arith_msu_tests() {
        let mut cpu = init_agc();
        let srcs = [
            (0o00002, 0o200, 0o00003, 0o177776),
            (0o00002, 0o200, 0o77776, 0o000004),
            (0o20000, 0o200, 0o30000, 0o167777),
            (0o00002, cpu::REG_Q, 0o00003, 0o177776),
            (0o00002, cpu::REG_Q, 0o177776, 0o000004),
            (0o20000, cpu::REG_Q, 0o30000, 0o167777),
        ];

        for (rega, idx, kval, expected) in srcs.iter() {
            let inst_data: u16 = 0o20000 + (*idx as u16);

            //println!("Testing MSU: {:o} {:o} {:o}", rega, kval, expected);
            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);

            cpu.reset();
            cpu.write(cpu::REG_A, *rega);
            cpu.write(*idx, *kval);

            cpu.step();
            validate_cpu_state(&mut cpu, 0x801);
            cpu.step();
            validate_cpu_state(&mut cpu, 0x802);

            assert_eq!(
                cpu.read(cpu::REG_A),
                *expected,
                "Failed MSU Test: A: {:06o} | K[{:o}] = {:06o} = e:{:06o}|a:{:06o}",
                rega,
                idx,
                kval,
                expected,
                cpu.read(cpu::REG_A)
            );
        }
    }

    // =========================================================================
    // SU instruction tests
    // =========================================================================
    #[test]
    fn arith_su_tests() {
        let mut cpu = init_agc();
        let srcs = [
            // Normal Subtraction
            (0o20000, 0o200, 0o77776, 0o20001),
            (0o20000, 0o200, 0o20000, 0o177777),
            (0o20000, 0o200, 0o20001, 0o177776),
            (0o20000, cpu::REG_Q, 0o177776, 0o20001),
            (0o20000, cpu::REG_Q, 0o070000, 0o127777),
            (0o20000, cpu::REG_Q, 0o020000, 0o177777),
            (0o20000, cpu::REG_Q, 0o020001, 0o177776),
            // Discovered this on Validate Test 16 / Subtest 10. SU was not
            // calculating to a proper value. Adding this as a unit test
            (0o170000, cpu::REG_Q, 0o00011, 0o167767),
        ];

        for (rega, idx, kval, expected) in srcs.iter() {
            let inst_data: u16 = 0o60000 + (*idx as u16);

            println!("Testing SU: {:o} {:o} {:o}", rega, kval, expected);
            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);

            cpu.reset();
            cpu.write(cpu::REG_A, *rega);
            cpu.write(*idx, *kval);

            cpu.step();
            validate_cpu_state(&mut cpu, 0x801);
            cpu.step();
            validate_cpu_state(&mut cpu, 0x802);

            assert_eq!(cpu.read(cpu::REG_A), *expected);
        }
    }

    // =========================================================================
    // MP instruction tests
    // =========================================================================
    #[test]
    fn arith_mp_tests() {
        let mut cpu = init_agc();
        let srcs = [
            // Normal Multiplication (A, K, Kval, ExpA, ExpL, ExpDouble)
            (0o00002, 0o200, 0o00002, 0o000000, 0o00004),
            (0o77775, 0o200, 0o77775, 0o000000, 0o00004),
            (0o00002, 0o200, 0o77775, 0o177777, 0o77773),
            (0o77775, 0o200, 0o00002, 0o177777, 0o77773),
            // Test Zero Case where it depends on which operand is what sign
            // to determine + or - 0
            (0o00000, 0o200, 0o00000, 0o000000, 0o00000), // Normal Positive Zero
            (0o77777, 0o200, 0o77777, 0o000000, 0o00000), // Positive Zero because
            // they are both zero
            (0o00000, 0o200, 0o77777, 0o000000, 0o00000), // Positive Zero because
            // they are both zero
            (0o77777, 0o200, 0o00000, 0o000000, 0o00000), // Positive Zero because
            // they are both zero
            (0o77777, 0o200, 0o00001, 0o177777, 0o77777), // Negative Zero because
            // A has the zero
            (0o00000, 0o200, 0o77776, 0o177777, 0o77777), // Negative Zero because
            // A has the zero
            (0o00001, 0o200, 0o77777, 0o000000, 0o00000), // Positive Zero because
            // A is not the zero
            (0o77776, 0o200, 0o00000, 0o000000, 0o00000), // Positive Zero because
                                                          // A is not the zero
        ];

        for (aval, k, kval, exp_aval, exp_lval) in srcs.iter() {
            let inst_data: u16 = 0o70000 + (*k as u16);
            println!(
                "Testing MP: {:o} {:o} {:o} {:o}",
                aval, kval, exp_aval, exp_lval
            );

            cpu.write(0x800, 0o00006);
            cpu.write(0x801, inst_data);

            cpu.reset();
            cpu.write_s15(cpu::REG_A, *aval);
            cpu.write_s15(*k, *kval);

            cpu.step();
            validate_cpu_state(&mut cpu, 0x801);
            cpu.step();
            validate_cpu_state(&mut cpu, 0x802);

            //assert_eq!(cpu.read(cpu::REG_A), *exp_aval);
            assert_eq!(cpu.read(cpu::REG_L), *exp_lval);
        }
    }
}
