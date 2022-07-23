use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use yaagc_protocol::agc::{parse_yaagc_packet, generate_yaagc_packet};

#[cfg(feature = "agc-net")]
use {
    embassy_net::tcp::TcpSocket,
    embassy_net::Stack,
    embedded_io::asynch::{Read, Write},
    crate::Device
};

use embassy::blocking_mutex::raw::ThreadModeRawMutex;
use embassy::channel::mpmc::Channel;

use embassy::util::{select, Either};
use heapless::spsc::{Consumer, Producer};

static YAAGC_TX_CHANNEL: Channel<ThreadModeRawMutex, (usize, u16), 32> = Channel::new();

fn handle_yaagc_message(keypress_tx: &mut Producer<'static, u16, 8>, channel: usize, value: u16) {
    match channel {
        0o15 => {
            //debug!("Keypress: {}", value);
            let _res = keypress_tx.enqueue(value);
        }
        0o32 => {
            //debug!("Keypress (Proceed): {}", value);
            let _res = keypress_tx.enqueue(value | 0o40000);
        }
        _ => {
            //warn!("Unimplemented keypress: {:?}", res);
        }
    }
}

pub async fn _send_yaagc_msg(channel: usize, value: u16) {
    let txchan_tx = YAAGC_TX_CHANNEL.sender();
    let res = txchan_tx.try_send((channel, value));
    match res {
        Ok(_) => {}
        Err(e) => {}
    }
}

#[embassy::task]
async fn monitor_dsky_msgs(mut dsky_msgs: Consumer<'static, (usize, u16), 64>) {
    let txchan_tx = YAAGC_TX_CHANNEL.sender();

    loop {
        while dsky_msgs.len() > 0 {
            let (chn, val) = dsky_msgs.dequeue().unwrap();
            txchan_tx.send((chn, val)).await;
        }
        Timer::after(Duration::from_micros(100)).await;
    }
}

#[embassy::task]
async fn flashing_lights(mut consumer: Consumer<'static, u16, 8>) {
    let mut channel_value = 0o00000;
    let start_time = embassy::time::Instant::now();
    let txchan_tx = YAAGC_TX_CHANNEL.sender();

    loop {
        while consumer.len() > 0 {
            channel_value = consumer.dequeue().unwrap();
        }

        let elapsed = start_time.elapsed().as_millis();
        if elapsed % 1000 < 750 {
            let mut value = channel_value;
            if channel_value & 0o00040 == 0o00040 {
                value &= !0o00040;
            }
            //dsky_tx.send(generate_yaagc_packet(0o0163, value)).await;
            //send_yaagc_msg(0o0163, value).await;
            txchan_tx.send((0o0163, value)).await;
        } else {
            if channel_value != 0o00000 {
                let mut value = channel_value & !0o00160;
                if channel_value & 0o00040 == 0o00040 {
                    value |= 0o00040;
                }
                //dsky_tx.send(generate_yaagc_packet(0o0163, value)).await;
                //send_yaagc_msg(0o0163, value).await;
                txchan_tx.send((0o0163, value)).await;
            }
        }

        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy::task]
async fn yaagc_dsky_socket_task(mut keypress_tx: Producer<'static, u16, 8>) -> ! {
    let mut parser = crate::tasks::parser::YaagcParser::new();
    let serial_recv = crate::tasks::serial::get_din();

    loop {
        // Flashing Channel Stuff
        let led = unsafe { crate::LED.steal() };
        let txchan_rx = YAAGC_TX_CHANNEL.receiver();
        loop {
            let a = serial_recv.recv();
            let b = txchan_rx.recv();
            match select(a, b).await {
                Either::First((msg, len)) => {
                    parser.push_bytes(&msg[..len]);
                    while parser.count() > 0 {
                        let (channel, value) = parser.pop_message().unwrap();
                        handle_yaagc_message(&mut keypress_tx, channel, value);
                        //led.toggle();
                    }
                },
                Either::Second((chn, val)) => {
                    led.toggle();
                    _ = crate::tasks::serial::send(&crate::tasks::parser::generate_packet(chn, val));
                }
            }
        }
    }
}

pub fn init_dsky_tasks(
    spawner: &Spawner,
    keypress_tx: Producer<'static, u16, 8>,
    dsky_rx: Consumer<'static, (usize, u16), 64>,
    flash_rx: Consumer<'static, u16, 8>,
) {
    let _ = spawner.spawn(yaagc_dsky_socket_task(keypress_tx));
    let _ = spawner.spawn(flashing_lights(flash_rx));
    let _ = spawner.spawn(monitor_dsky_msgs(dsky_rx));
}
