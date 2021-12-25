#[cfg(feature = "std")]
#[cfg(test)]
mod scalar_tests {
    use crate::cpu;
    use crate::instr::tests::{init_agc, validate_cpu_state};

    #[test]
    fn scalar_read_test() {
        let mut cpu = init_agc();

        for scalar_count in 1..5 {
            // Write a bunch of noops to generate 1MCTs in order to
            // increment the internal SCALAR value.
            for i in 0..27 * scalar_count {
                cpu.write(0x800 + i, 0o10000 + 0x801 + i as u16);
            }
            cpu.write(0x800 + 27, 0o00006); // EXTEND
            cpu.write(0x800 + 28, 0o00004); // RAND LOSCALAR
            cpu.reset();

            for i in 0..29 {
                cpu.step();
                validate_cpu_state(&mut cpu, 0x801 + i as u16);
            }

            assert_eq!(scalar_count as u16, cpu.read_io(4));
            assert_eq!(scalar_count as u16, cpu.read(cpu::REG_A));
        }
    }

    ///
    /// ## TIMER_INCREMENT_TEST:
    ///
    /// The following test validates that after a full cycle of SCALAR increments
    /// that majority of the TIME registers were updated, and validate the increment
    /// through the use of the `read` function
    ///
    #[test]
    fn timer_increment_test() {
        let mut cpu = init_agc();
        let scalar_count = 32;

        // Write a bunch of noops to generate 1MCTs in order to
        // increment the internal clocks.
        for i in 0..27 {
            cpu.write(0x800 + i, 0o10000 + 0x801 + i as u16);
        }
        cpu.write(0x800 + 26, 0o10000 + 0x800);
        cpu.reset();

        for i in 0..27 * scalar_count {
            let current_pc = cpu.read(cpu::REG_Z);
            loop {
                cpu.step();
                if current_pc != cpu.read(cpu::REG_Z) {
                    break;
                }
            }
            validate_cpu_state(&mut cpu, 0x800 + ((1 + i as u16) % 27));
        }

        assert_eq!(1, cpu.read(crate::mem::timer::MM_TIME1));
        assert_eq!(0, cpu.read(crate::mem::timer::MM_TIME2));
        assert_eq!(1, cpu.read(crate::mem::timer::MM_TIME3));
        assert_eq!(1, cpu.read(crate::mem::timer::MM_TIME4));
        assert_eq!(1, cpu.read(crate::mem::timer::MM_TIME5));
        //assert_eq!(scalar_count as u16, cpu.read(cpu::REG_A));
    }

    ///
    /// ## TIME Interrupt Test:
    ///
    /// The following test validates that a timer interrupt has occurred on a
    /// proper overflow of the specific timer. The test ensures that after X amount
    /// of steps, the PC will jump to the specific interrupt location.
    ///

    #[test]
    fn timer_interrupt_test() {
        let scalar_count = 32;
        let setup = [
            (crate::mem::timer::MM_TIME3, 0x80C),
            (crate::mem::timer::MM_TIME4, 0x810),
            (crate::mem::timer::MM_TIME5, 0x808),
        ];

        for (timer_addr, interrupt_addr) in setup.iter() {
            println!("Testing {:?} interrupt", timer_addr);
            let mut cpu = init_agc();

            // Write a bunch of noops to generate 1MCTs in order to
            // increment the internal clocks.
            for i in 0..27 {
                cpu.write(0x900 + i, 0o10000 + 0x901 + i as u16);
            }
            cpu.write(0x900 + 26, 0o10000 + 0x900);
            cpu.write(0x800, 0o10000 + 0x900);
            cpu.reset();
            cpu.write(*timer_addr, 0o77777);
            cpu.step();
            validate_cpu_state(&mut cpu, 0x900);

            for i in 0..27 * scalar_count {
                let current_pc = cpu.read(cpu::REG_Z);
                loop {
                    cpu.step();
                    if current_pc != cpu.read(cpu::REG_Z) {
                        break;
                    }
                }

                let pc = cpu.read(cpu::REG_Z);
                if pc == *interrupt_addr {
                    assert_eq!(0o00000, cpu.read(crate::mem::timer::MM_TIME3));
                    break;
                } else {
                    validate_cpu_state(&mut cpu, 0x900 + ((1 + i as u16) % 27));
                }
            }
        }
    }

    ///
    /// ## TIME Interrupt Priority Test:
    ///
    /// The following test validates that a timer interrupt has occurred on a
    /// proper overflow of the specific timer. The test ensures that after X amount
    /// of steps, the PC will jump to the specific interrupt location.
    ///

    #[test]
    fn timer_interrupt_priority_test() {
        let scalar_count = 32;
        let setup = [
            (crate::mem::timer::MM_TIME3, 0x80C),
            (crate::mem::timer::MM_TIME4, 0x810),
            (crate::mem::timer::MM_TIME5, 0x808),
        ];

        for (timer_addr, interrupt_addr) in setup.iter() {
            println!("Testing {:?} interrupt", timer_addr);
            let mut cpu = init_agc();

            // Write a bunch of noops to generate 1MCTs in order to
            // increment the internal clocks.
            for i in 0..27 {
                cpu.write(0x900 + i, 0o10000 + 0x901 + i as u16);
            }
            cpu.write(0x900 + 26, 0o10000 + 0x900);
            cpu.write(0x800, 0o10000 + 0x900);
            cpu.reset();
            cpu.write(*timer_addr, 0o77777);
            cpu.step();
            validate_cpu_state(&mut cpu, 0x900);

            for i in 0..27 * scalar_count {
                let current_pc = cpu.read(cpu::REG_Z);
                loop {
                    cpu.step();
                    if current_pc != cpu.read(cpu::REG_Z) {
                        break;
                    }
                }

                let pc = cpu.read(cpu::REG_Z);
                if pc == *interrupt_addr {
                    assert_eq!(0o00000, cpu.read(crate::mem::timer::MM_TIME3));
                    break;
                } else {
                    validate_cpu_state(&mut cpu, 0x900 + ((1 + i as u16) % 27));
                }
            }
        }
    }
}
