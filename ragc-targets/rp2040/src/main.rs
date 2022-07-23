#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_rp::Peripherals;
use embassy_rp::gpio::{Output, Level};

use embassy_rp::peripherals::PIN_25;
use panic_probe as _;

mod tasks;

pub static LED: Forever<Output<PIN_25>> = Forever::new();

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    let led = Output::new(p.PIN_25, Level::Low);
    LED.put(led);

    tasks::agc::init_agc_tasks(&spawner);
    tasks::serial::init_usb_serial_task(&spawner);

    loop {
        Timer::after(Duration::from_secs(5)).await;
    }
}
