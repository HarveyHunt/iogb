extern crate argparse;

use std::path::PathBuf;
use argparse::{ArgumentParser, Parse, Print};

mod gb;
mod cpu;
mod mmu;
mod cartridge;

fn main() {
    let mut rom = PathBuf::new();

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("A GameBoy emulator written in Rust");
        parser.add_option(&["-v", "--version"],
                          Print(format!("iogb: v{}", env!("CARGO_PKG_VERSION"))),
                          "Show version");
        parser.refer(&mut rom).add_option(&["-r", "--rom"], Parse, "Path to ROM file").required();
        parser.parse_args_or_exit();
    }

    let cart = cartridge::Cartridge::new(rom);
    let mut gb = gb::GameBoy::new(cart);

    println!("gb: {:?}", gb);

    gb.run();
}
