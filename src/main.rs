#![deny(trivial_casts, trivial_numeric_casts)]
extern crate argparse;

#[macro_use]
extern crate bitflags;
extern crate time;

extern crate minifb;

use std::path::PathBuf;
use std::process;
use time::{SteadyTime, Duration};
use argparse::{ArgumentParser, Parse, Print};
use minifb::{Key, WindowOptions, Window};

use gameboy::{SCREEN_W, SCREEN_H};

mod gameboy;
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

    let mut window = Window::new("iogb", 160, 144, WindowOptions::default()).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut gb = gameboy::GameBoy::new(cart, bootrom);
    let mut ticks = 0;
    let mut delta: Duration;
    let mut last_time = SteadyTime::now();

    let mut buffer: Vec<u32> = vec![0; SCREEN_W * SCREEN_H];

    while window.is_open() {
        let now = SteadyTime::now();
        delta = now - last_time;
        last_time = now;

        // TODO: Receive VSYNC event so we can regenerate the texture
        // from the GPU's back buffer.
        gb.run((delta * gameboy::CPU_HZ as i32).num_seconds() as u32);

        // Convert from pixels in range 0..3 to full colours.
        for (i, pixel) in gb.back_buffer().iter().enumerate() {
            buffer[i] = (3 - *pixel as u32) * 0x404040;
        }
        window.update_with_buffer(&buffer[..]);
    }
}
