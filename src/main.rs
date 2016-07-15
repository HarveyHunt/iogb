#![deny(trivial_casts, trivial_numeric_casts)]
extern crate argparse;

#[macro_use]
extern crate bitflags;

use std::path::PathBuf;
use std::process;
use argparse::{ArgumentParser, Parse, Print};

mod gb;
mod cpu;
mod interconnect;
mod cartridge;
mod interrupt;
mod timer;
mod gpu;
mod bootrom;

fn main() {
    let mut rom = PathBuf::new();
    let mut bootrom = PathBuf::new();

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("A GameBoy emulator written in Rust");
        parser.add_option(&["-v", "--version"],
                          Print(format!("iogb: v{}", env!("CARGO_PKG_VERSION"))),
                          "Show version");
        parser.refer(&mut rom).add_option(&["-r", "--rom"], Parse, "Path to ROM file").required();
        parser.refer(&mut bootrom)
            .add_option(&["-b", "--bootrom"], Parse, "Path to boot ROM file")
            .required();
        parser.parse_args_or_exit();
    }

    let cart = match cartridge::Cartridge::new(&rom) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to open cartridge: {} {}", rom.display(), e);
            process::exit(1)
        }
    };

    let bootrom = match bootrom::load_bootrom(&bootrom) {
        Ok(b) => b,
        Err(e) => {
            println!("Failed to load bootrom: {} {}", bootrom.display(), e);
            process::exit(1)
        }
    };

    let mut gb = gb::GameBoy::new(cart, bootrom);

    gb.run();
}
