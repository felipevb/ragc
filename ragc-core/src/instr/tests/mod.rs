use crate::cpu;
use crate::cpu::AgcCpu;
use crate::mem;
use heapless::spsc::Queue;
use ragc_ropes;

#[allow(dead_code)]
pub fn init_agc<'a>() -> AgcCpu<'a> {
    let mut rupt_queue = Queue::new();
    let (rupt_tx, _rupt_rx) = rupt_queue.split();

    let program = ragc_ropes::LUMINARY131_ROPE;

    let mut mm = mem::AgcMemoryMap::new(program ,rupt_tx);
    mm.enable_rom_write();
    let mut _cpu = cpu::AgcCpu::new(mm);

    _cpu
}

#[allow(dead_code)]
pub fn validate_cpu_state(cpu: &mut AgcCpu, expect_pc: u16) {
    assert_eq!(cpu.read(cpu::REG_Z), expect_pc);
}

mod ad;
mod arith;
mod logic;
