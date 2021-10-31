extern crate clap;

use crossbeam_channel::bounded;
use ctrlc;
use env_logger;
use log::error;

use ragc::{cpu, mem};

fn fetch_config<'a>() -> clap::ArgMatches<'a> {
    let about =
        "RAGC is a Block-II Apollo Guidance Computer functional emulator written entirely in Rust";
    let c = clap::App::new("Rust Apollo Guidance Computer (RAGC)")
        .version("0.1")
        .about(about)
        .arg(
            clap::Arg::with_name("input")
                .required(true)
                .help("Input Firmware File to Run"),
        );
    let a = c.get_matches();
    a
}

fn main() {
    env_logger::init();
    //env_logger::Builder::new()
    //    .target(env_logger::Target::Stdout)
    //    .format(|buf, record| {
    //        writeln!(buf, "[{}] - {}", record.level(), record.args())
    //    })
    //    .filter(None, LevelFilter::Debug)
    //    .init();

    // Register for a ctrlc handler which will push a signal to the application.
    // If the signal handler is pushed multiple times without closing, then force
    // closing the application and lose any close-ups of
    let (ctrlc_tx, ctrlc_rx) = bounded(1);
    let res = ctrlc::set_handler(move || {
        if ctrlc_tx.is_full() == true {
            std::process::exit(-1);
        }
        let _res = ctrlc_tx.send(());
    });

    match res {
        Err(x) => {
            error!("Unable to register signal handler. {:?}.", x);
            return;
        }
        _ => {}
    }

    let matches = fetch_config();
    let filename = matches.value_of("input").unwrap();

    let mut q1 = heapless::spsc::Queue::new();
    let (rupt_tx, _rupt_rx) = q1.split();

    let mm = mem::AgcMemoryMap::new(&filename, rupt_tx);
    let mut _cpu = cpu::AgcCpu::new(mm);

    _cpu.reset();
    let mut last_timestamp = std::time::Instant::now();
    loop {
        // Check to see if we received a ctrlc signal. If we have, we need to
        // exit out of the loop and exit the application.
        if ctrlc_rx.len() > 0 {
            break;
        }

        if last_timestamp.elapsed().as_millis() == 0 {
            std::thread::sleep(std::time::Duration::new(0, 5000000));
            continue;
        }

        let mut cycle_counter = 0;
        let expected_cycles = ((last_timestamp.elapsed().as_micros() as f64) / 11.7) as i64;
        while cycle_counter < expected_cycles {
            cycle_counter += _cpu.step() as i64;
        }
        last_timestamp = std::time::Instant::now();
    }
}
