#[cfg(test)]
mod ad_tests {
    use crate::cpu;
    use crate::instr::tests::init_agc;
    use crate::instr::tests::validate_cpu_state;

    #[test]
    fn ad_add() {
        let mut cpu = init_agc();

        // Performing basic Add with AD instruction
        // (Register + Register) to demonstrate this works
        let mut inst_data: u16 = 0o60000 + cpu::REG_L as u16;
        cpu.write(0x800, inst_data);
        cpu.reset();

        cpu.write(cpu::REG_A, 0x10);
        cpu.write(cpu::REG_L, 0x10);
        cpu.step();

        let mut result = cpu.read(cpu::REG_A);
        validate_cpu_state(&mut cpu, 0x801);
        assert_eq!(result, 0x20);

        // Performing basic Add with AD instruction
        // (Register + Mem) to demonstrate this works
        inst_data = 0o60000 + 0o200 as u16;
        cpu.write(0x800, inst_data);
        cpu.reset();

        cpu.write(cpu::REG_A, 0x10);
        cpu.write(0o200, 0x10);
        cpu.step();

        result = cpu.read(cpu::REG_A);
        validate_cpu_state(&mut cpu, 0x801);
        assert_eq!(result, 0x20);
    }

    #[test]
    fn ad_overflow() {
        let mut cpu = init_agc();

        // Create a test to handle overflow (reg/reg)
        // so from A register to another general purpose
        // register
        let mut inst_data: u16 = 0o60000 + cpu::REG_L as u16;
        cpu.write(0x800, inst_data);
        cpu.reset();

        cpu.write(cpu::REG_A, 0x3FFF);
        cpu.write(cpu::REG_L, 0x0001);
        cpu.step();

        let mut result = cpu.read(cpu::REG_A);
        validate_cpu_state(&mut cpu, 0x801);
        assert_eq!(result, 0x4000);

        // Create a test to handle overflow (reg/mem)
        // so from A register and a general RAM memory
        // location
        inst_data = 0o60000 + 0o200 as u16;
        cpu.write(0x800, inst_data);
        cpu.reset();

        cpu.write(cpu::REG_A, 0x3FFF);
        cpu.write(0o200, 0x0001);
        cpu.step();

        result = cpu.read(cpu::REG_A);
        validate_cpu_state(&mut cpu, 0x801);
        assert_eq!(result, 0x4000);
    }

    #[test]
    fn ad_underflow() {
        let mut cpu = init_agc();

        // Create a test to handle overflow (reg/reg)
        // so from A register to another general purpose
        // register
        let mut inst_data: u16 = 0o60000 + cpu::REG_L as u16;
        cpu.write(0x800, inst_data);
        cpu.reset();

        cpu.write(cpu::REG_A, 0xC000);
        cpu.write(cpu::REG_L, 0xFFFE);
        cpu.step();

        let mut result = cpu.read(cpu::REG_A);
        validate_cpu_state(&mut cpu, 0x801);
        assert_eq!(result, 0xBFFF);

        // Create a test to handle overflow (regs/mem)
        // so from A register and a general RAM memory
        // location
        inst_data = 0o60000 + 0o200 as u16;
        cpu.write(0x800, inst_data);
        cpu.reset();

        cpu.write(cpu::REG_A, 0xC000);
        cpu.write(0o200, 0xFFFE);
        cpu.step();

        result = cpu.read(cpu::REG_A);
        validate_cpu_state(&mut cpu, 0x801);
        assert_eq!(result, 0xBFFF);
    }

    #[test]
    fn ad_test_sub1() {
        let mut cpu = init_agc();

        let inst_data: u16 = 0o60000 + cpu::REG_L as u16;
        cpu.write(0x800, inst_data);
        cpu.reset();

        cpu.write(cpu::REG_A, 0xFFFE);
        cpu.write(cpu::REG_L, 0xFFFE);
        cpu.step();

        let result = cpu.read(cpu::REG_A);
        validate_cpu_state(&mut cpu, 0x801);
        print!("Result {:x}", result);
        assert_eq!(result, 0xFFFD);
    }

    #[test]
    fn ad_test_sub2() {
        let mut cpu = init_agc();

        let inst_data: u16 = 0o60000 + cpu::REG_L as u16;
        cpu.write(0x800, inst_data);
        cpu.reset();

        cpu.write(cpu::REG_A, 0x0020);
        cpu.write(cpu::REG_L, 0xFFFD);
        cpu.step();

        let result = cpu.read(cpu::REG_A);
        validate_cpu_state(&mut cpu, 0x801);
        print!("Result {:x}", result);
        assert_eq!(result, 0x1E);
    }

    #[test]
    fn ads_add() {
        let mut cpu = init_agc();

        // Performing basic Add with ADS instruction
        // (Register + Register) to demonstrate this works
        let mut inst_data: u16 = 0o26000 + cpu::REG_L as u16;
        cpu.write(0x800, inst_data);
        cpu.reset();

        cpu.write(cpu::REG_A, 0x10);
        cpu.write(cpu::REG_L, 0x10);
        cpu.step();

        let mut result = cpu.read(cpu::REG_A);
        let mut result1 = cpu.read(cpu::REG_L);
        validate_cpu_state(&mut cpu, 0x801);
        assert_eq!(result, 0x20);
        assert_eq!(result1, 0x20);

        // Performing basic Add with AD instruction
        // (Register + Mem) to demonstrate this works
        inst_data = 0o26000 + 0o200 as u16;
        cpu.write(0x800, inst_data);
        cpu.reset();

        cpu.write(cpu::REG_A, 0x10);
        cpu.write(0o200, 0x10);
        cpu.step();

        result = cpu.read(cpu::REG_A);
        result1 = cpu.read(0o200);
        validate_cpu_state(&mut cpu, 0x801);
        assert_eq!(result, 0x20);
        assert_eq!(result1, 0x20);
    }

    #[test]
    fn ads_overflow() {
        let mut cpu = init_agc();
        let srcs = [cpu::REG_L, 0o200];

        // Create a test to handle overflow (reg/reg)
        // so from A register to another general purpose
        // register
        for i in srcs.iter() {
            let idx = *i;
            let inst_data: u16 = 0o26000 + idx as u16;
            cpu.write(0x800, inst_data);
            cpu.reset();

            cpu.write(cpu::REG_A, 0x3FFF);
            cpu.write(idx, 0x0001);
            cpu.step();

            validate_cpu_state(&mut cpu, 0x801);
            assert_eq!(cpu.read(cpu::REG_A), 0x4000);
            assert_eq!(cpu.read(idx), 0x0000);
            assert_eq!(cpu.read_s16(idx), 0x0000);
        }
    }

    #[test]
    fn ads_underflow() {
        let mut cpu = init_agc();
        let srcs = [cpu::REG_L, 0o200];

        for i in srcs.iter() {
            // Create a test to handle overflow (reg/reg)
            // so from A register to another general purpose
            // register
            let idx = *i;
            let inst_data: u16 = 0o26000 + idx as u16;
            cpu.write(0x800, inst_data);
            cpu.reset();

            cpu.write(cpu::REG_A, 0xC000);
            cpu.write(idx, 0xFFFE);
            cpu.step();

            validate_cpu_state(&mut cpu, 0x801);
            assert_eq!(cpu.read(cpu::REG_A), 0xBFFF);
            assert_eq!(cpu.read(idx), 0x7FFF);
            assert_eq!(cpu.read_s16(idx), 0xFFFF);
        }
    }

    #[test]
    fn ads_sub1() {
        let mut cpu = init_agc();
        let srcs = [cpu::REG_L, 0o200];

        for i in srcs.iter() {
            let idx = *i;
            let inst_data: u16 = 0o26000 + idx as u16;

            cpu.write(0x800, inst_data);
            cpu.reset();

            cpu.write(cpu::REG_A, 0xFFFE);
            cpu.write(idx, 0xFFFE);
            cpu.step();

            validate_cpu_state(&mut cpu, 0x801);
            assert_eq!(cpu.read(cpu::REG_A), 0xFFFD);
            assert_eq!(cpu.read(idx), 0x7FFD);
            assert_eq!(cpu.read_s16(idx), 0xFFFD);
        }
    }

    #[test]
    fn ads_sub2() {
        let mut cpu = init_agc();
        let srcs = [cpu::REG_L, 0o200];

        for i in srcs.iter() {
            let idx = *i;
            let inst_data: u16 = 0o26000 + idx as u16;

            cpu.write(0x800, inst_data);
            cpu.reset();

            cpu.write(cpu::REG_A, 0x20);
            cpu.write(idx, 0xFFFD);
            cpu.step();

            validate_cpu_state(&mut cpu, 0x801);
            assert_eq!(cpu.read(cpu::REG_A), 0x1E);
            assert_eq!(cpu.read(idx), 0x1E);
            assert_eq!(cpu.read_s16(idx), 0x1E);
        }
    }
}
