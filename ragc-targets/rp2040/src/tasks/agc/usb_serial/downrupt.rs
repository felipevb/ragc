use embassy::time::{Duration, Timer};
use heapless::spsc::Consumer;

#[embassy::task]
pub async fn downrupt_serial_task(
    mut _downrupt_rx: Consumer<'static, (usize, u16), 4>,
) -> ! {

    loop {
        Timer::after(Duration::from_millis(10)).await;
    }
}
