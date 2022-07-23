use embassy::{
    executor::Spawner,
    util::Forever,
};

use heapless::spsc::Queue;
use ragc_periph::downrupt::DownruptPeriph;

use super::usb_serial::downrupt;

static DOWNRUPT_QUEUE: Forever<Queue<(usize, u16), 4>> = Forever::new();

pub fn init_downrupt_tasks<'a>(
    spawner: &Spawner,
) -> DownruptPeriph<'a> {
    let q = DOWNRUPT_QUEUE.put(Queue::new());
    let (downrupt_tx, downrupt_rx) = q.split();

    let downrupt = DownruptPeriph::new(downrupt_tx);
    let _ = spawner.spawn(downrupt::downrupt_serial_task(downrupt_rx));

    downrupt
}
