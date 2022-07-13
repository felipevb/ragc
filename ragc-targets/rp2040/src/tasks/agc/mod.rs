use embassy::time::{Duration, Instant, Timer};
use embassy::{executor::Spawner, util::Forever};

#[cfg(feature = "agc-net")]
use embassy_net::Stack;

use heapless::spsc::Queue;
use ragc_periph::downrupt::DownruptPeriph;
use ragc_periph::dsky::DskyDisplay;
use ragc_ropes::RETREAD50_ROPE as ROPE;

#[cfg(feature = "agc-net")]
use crate::Device;

mod downrupt;
mod dsky;

static KEYPRESS_QUEUE: Forever<Queue<u16, 8>> = Forever::new();
static DSKY_QUEUE: Forever<Queue<(usize, u16), 64>> = Forever::new();
static FLASH_QUEUE: Forever<Queue<u16, 8>> = Forever::new();

#[embassy::task]
async fn agc_cpu_task(mut downrupt: DownruptPeriph<'static>, mut dsky: DskyDisplay<'static>) {
    let mut q = Queue::new();
    let (rupt_tx, rupt_rx) = q.split();

    let memmap = ragc_core::mem::AgcMemoryMap::new(ROPE, &mut downrupt, &mut dsky, rupt_tx);
    let mut cpu = ragc_core::cpu::AgcCpu::new(memmap);

    cpu.reset();
    loop {
        let s = Instant::now();
        for i in 0..50 {
            cpu.step();
        }
        let duration = 117 * 5;
        Timer::after(Duration::from_micros(duration)).await;
    }
}

pub fn init_agc_tasks(spawner: &Spawner) {
    let keypress_queue = KEYPRESS_QUEUE.put(Queue::new());
    let (kp_tx, mut kp_rx) = keypress_queue.split();

    let dsky_queue = DSKY_QUEUE.put(Queue::new());
    let (dsky_tx, dsky_rx) = dsky_queue.split();

    let flash_queue = FLASH_QUEUE.put(Queue::new());
    let (flash_tx, flash_rx) = flash_queue.split();

    dsky::init_dsky_tasks(spawner, kp_tx, dsky_rx, flash_rx);
    let downrupt = downrupt::init_downrupt_tasks(spawner);
    let dsky = DskyDisplay::new(kp_rx, dsky_tx, flash_tx);
    spawner.spawn(agc_cpu_task(downrupt, dsky));
}
