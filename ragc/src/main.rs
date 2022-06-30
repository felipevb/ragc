extern crate clap;

use crossbeam_channel::bounded;
use ctrlc;
use env_logger;
use log::error;

use ragc_core::{cpu, mem};
use ragc_ropes;
use ragc_periph;

pub const ROM_BANKS_NUM: usize = 36;
pub const ROM_BANK_NUM_WORDS: usize = 1024;

use std::io::Read;
use std::fs::File;

fn fetch_config<'a>() -> clap::ArgMatches<'a> {
    let about =
        "RAGC is a Block-II Apollo Guidance Computer functional emulator written entirely in Rust";
    let c = clap::App::new("Rust Apollo Guidance Computer (RAGC)")
        .version("0.1")
        .about(about)
        .subcommand(
            clap::SubCommand::with_name("retread50")
                .help("Run AGC with RETREAD50 ROM and Configuration")
        )
        .subcommand(
            clap::SubCommand::with_name("validation")
                .help("Run AGC with VALIDATION ROM")
        )
        .subcommand(
            clap::SubCommand::with_name("luminary131")
                .help("Run AGC with LUMINARY131 ROM and Configuration")
        )
        .subcommand(
            clap::SubCommand::with_name("file")
                .help("Run ROM from agcbin file")
                .arg(clap::Arg::with_name("filename")
                    .index(1)
                    .help("Filename of agcbin to load")
            )
        );
    let a = c.get_matches();
    a
}

fn load_agcbin_file(filename: &str) -> Option<[[u16; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM]> {
    // Check to make sure we are able to open the file. If we are not
    // able to, throw up the issue up to the caller to know we failed
    // at opening the file.
    let fp = File::open(filename);
    let mut f = match fp {
        Ok(f) => f,
        _ => {
            error!("Unable to open file: {:?}", filename);
            return None;
        }
    };

    let mut buf = [0; ROM_BANK_NUM_WORDS * 2];
    let mut banks = [[0; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM];

    let mut bank_idx = 0;
    loop {
        match f.read_exact(&mut buf) {
            Ok(_x) => {
                let mut word_idx = 0;
                for c in buf.chunks_exact(2) {
                    let res = (c[1] as u16) << 8 | c[0] as u16;
                    banks[bank_idx][word_idx] = res; //res >> 1;
                    word_idx += 1;
                }
            }
            Err(_x) => {
                break;
            }
        };
        bank_idx += 1;
    }

    Some(banks)
}

fn main() {
    env_logger::init();

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
    let rope = match matches.subcommand_name() {
        Some("retread50") => {
            *ragc_ropes::RETREAD50_ROPE
        }
        Some("luminary131") => {
            *ragc_ropes::LUMINARY131_ROPE
        }
        Some("validation") => {
            *ragc_ropes::VALIDATION_ROPE
        }
        Some("file") => {
            let sub_matches = matches.subcommand_matches("file").unwrap();
            let filename = sub_matches.value_of("filename").unwrap();
            load_agcbin_file(&filename).unwrap()
        }
        _ => {
            error!("Invalid subcommand. Exiting");
            return
        }
    };

    let mut q1 = heapless::spsc::Queue::new();
    let (rupt_tx, _rupt_rx) = q1.split();

    let mut dsky = ragc_periph::dsky::DskyDisplay::new();
    let mut downrupt = ragc_periph::downrupt::DownruptPeriph::new();

    let mm = mem::AgcMemoryMap::new(&rope, &mut downrupt, &mut dsky, rupt_tx);
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
