use crate::cpu;
use crate::cpu::AgcCpu;
use crate::mem;
use heapless::spsc::Queue;

#[allow(dead_code)]
pub fn init_agc() -> AgcCpu {
    let mut rupt_queue = Queue::new();
    let (rupt_tx, _rupt_rx) = rupt_queue.split();

    let mut mm = mem::AgcMemoryMap::new_blank(rupt_tx);
    mm.enable_rom_write();
    let mut _cpu = cpu::AgcCpu::new(mm);

    _cpu
}

#[allow(dead_code)]
pub fn validate_cpu_state(cpu: &mut AgcCpu, expect_pc: u16) {
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
