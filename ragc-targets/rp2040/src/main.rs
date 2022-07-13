#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_rp::{gpio, Peripherals};
use gpio::{Level, Output};
use panic_probe as _;

mod tasks;

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    let mut led = Output::new(p.PIN_25, Level::Low);

    tasks::agc::init_agc_tasks(&spawner);
    loop {
        led.set_high();
        Timer::after(Duration::from_secs(1)).await;

        led.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
}
