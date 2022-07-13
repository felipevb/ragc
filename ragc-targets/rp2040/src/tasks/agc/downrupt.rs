use embassy::{
    executor::Spawner,
    time::{Duration, Timer},
    util::Forever,
};

#[cfg(feature = "agc-net")]
use {
    embassy_net::{tcp::TcpSocket, Stack},
    crate::{utils::generate_yaagc_packet, Device}
};

//use embedded_io::asynch::Write;
use heapless::spsc::{Consumer, Queue};
use ragc_periph::downrupt::DownruptPeriph;

static DOWNRUPT_QUEUE: Forever<Queue<(usize, u16), 4>> = Forever::new();

#[embassy::task]
async fn downrupt_network_task(
    mut downrupt_rx: Consumer<'static, (usize, u16), 4>,
) -> ! {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    loop {
        loop {
            while downrupt_rx.len() > 0 {
                let a = downrupt_rx.dequeue();
                match a {
                    Some(x) => {
                        ;
                    }
                    None => {}
                }
            }
            Timer::after(Duration::from_millis(10)).await;
        }
    }
}

pub fn init_downrupt_tasks<'a>(
    spawner: &Spawner,
) -> DownruptPeriph<'a> {
    let q = DOWNRUPT_QUEUE.put(Queue::new());
    let (downrupt_tx, downrupt_rx) = q.split();

    let downrupt = DownruptPeriph::new(downrupt_tx);
    spawner.spawn(downrupt_network_task(downrupt_rx));

    downrupt
}
