#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_rp::Peripherals;
//use gpio::{Level, Output};
use panic_probe as _;

use rp2040_hal as hal;
use hal::pac;

use embedded_hal::digital::v2::OutputPin;

mod tasks;

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {

    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    //let mut led = Output::new(p.PIN_25, Level::Low);
    tasks::agc::init_agc_tasks(&spawner);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure GPIO25 as an output
    let mut led_pin = pins.gpio25.into_push_pull_output();
    loop {
        led_pin.set_high().unwrap();
        Timer::after(Duration::from_secs(1)).await;
        led_pin.set_low().unwrap();
        Timer::after(Duration::from_secs(1)).await;
    }
}
