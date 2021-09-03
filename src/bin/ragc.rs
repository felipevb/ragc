extern crate clap;

use crossbeam_channel::unbounded;
use env_logger;

use ragc::{mem, cpu};


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
    let matches = fetch_config();
    let filename = matches.value_of("input").unwrap();

    let (rupt_tx, _rupt_rx) = unbounded();
    let (incr_tx, incr_rx) = unbounded();

    let mm = mem::AgcMemoryMap::new(&filename, rupt_tx, incr_rx);
    let mut _cpu = cpu::AgcCpu::new(mm, incr_tx);

    _cpu.reset();
    let mut last_timestamp = std::time::Instant::now();
    loop {
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
