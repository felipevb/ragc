use crate::cpu;
use crate::cpu::AgcCpu;
use crate::mem;
use crossbeam_channel::unbounded;

#[allow(dead_code)]
pub fn init_agc() -> AgcCpu {
    let (rupt_tx, _rupt_rx) = unbounded();
    let (incr_tx, incr_rx) = unbounded();

    let mut mm = mem::AgcMemoryMap::new_blank(rupt_tx, incr_rx);
    mm.enable_rom_write();
    //let mut iospace = mem::AgcIoSpace::new(mm.clone());
    //let mut _cpu = cpu::AgcCpu::new(mm, iospace, incr_tx);
    let mut _cpu = cpu::AgcCpu::new(mm, incr_tx);

    _cpu
}

#[allow(dead_code)]
pub fn validate_cpu_state(cpu: &AgcCpu, expect_pc: u16) {
    assert_eq!(cpu.read(cpu::REG_Z), expect_pc);
}

mod init_tests {
    #[test]
    fn helloworld() {
        env_logger::init();
    }
}

mod ad;
mod arith;
mod logic;
