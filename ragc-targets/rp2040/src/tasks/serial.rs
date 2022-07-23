use embassy::channel::mpmc::{Channel, Receiver};
use embassy::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy::executor::Spawner;

use embassy_rp::interrupt;

use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

use rp2040_hal as hal;

use hal::clocks::ClocksManager;
use hal::pac::{self, Peripherals};

const PACKET_SIZE: usize = 64;
const USB_PACKET_RX_QUEUE_SIZE: usize = 16;

static INTR_SERIAL_DIN:  Channel<CriticalSectionRawMutex, ([u8; PACKET_SIZE], usize), USB_PACKET_RX_QUEUE_SIZE> = Channel::new();

/// The USB Device Driver (shared with the interrupt).
static mut USB_DEVICE: Option<UsbDevice<hal::usb::UsbBus>> = None;

/// The USB Bus Driver (shared with the interrupt).
static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;

/// The USB Serial Device Driver (shared with the interrupt).
static mut USB_SERIAL: Option<SerialPort<hal::usb::UsbBus>> = None;


pub fn get_din() -> Receiver<'static, CriticalSectionRawMutex, ([u8; PACKET_SIZE], usize), USB_PACKET_RX_QUEUE_SIZE> {
    INTR_SERIAL_DIN.receiver()
}

pub fn send(data: &[u8]) -> Result<usize, UsbError> {
    unsafe {
        pac::NVIC::mask(hal::pac::Interrupt::USBCTRL_IRQ);
        let usb_serial = USB_SERIAL.as_mut().unwrap();
        let result = usb_serial.write(data);
        pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);
        result
    }
}

pub fn init_usb_serial_task(_spawner: &Spawner)  {
    let mut pac = Peripherals::take().unwrap();

    // Set up the USB driver
    let clocks = ClocksManager::new(pac.CLOCKS);
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let usb_ref = unsafe {
        USB_BUS = Some(usb_bus);
        USB_BUS.as_ref().unwrap()
    };


    // Set up the USB Communications Class Device driver
    let serial = SerialPort::new(usb_ref);

    // Create a USB device with a fake VID and PID
    let usb_dev = UsbDeviceBuilder::new(usb_ref, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(2)
        .build();

    unsafe {
        USB_SERIAL = Some(serial);
        USB_DEVICE = Some(usb_dev);
    };

    // Enable the USB interrupt
    unsafe {
        pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);
    };
}

#[interrupt]
unsafe fn USBCTRL_IRQ() {
    // Grab the global objects. This is OK as we only access them under interrupt.
    let usb_dev = USB_DEVICE.as_mut().unwrap();
    let serial = USB_SERIAL.as_mut().unwrap();

    let din = INTR_SERIAL_DIN.sender();

    // Poll the USB driver with all of our supported USB Classes
    if usb_dev.poll(&mut [serial]) {
        let mut buf = [0u8; PACKET_SIZE];
        match serial.read(&mut buf) {
            Err(_e) => {
                // Do nothing
            }
            Ok(0) => {
                // Do nothing
            }
            Ok(count) => {
                _ = din.try_send((buf, count));
            }
        }
    }
}
