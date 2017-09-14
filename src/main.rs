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
use minifb::{WindowOptions, Window, Scale};

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
    let mut bootrom_path = PathBuf::new();
    let mut scale: u32 = 1;

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("A GameBoy emulator written in Rust");
        parser.add_option(&["-v", "--version"],
                          Print(format!("iogb: v{}", env!("CARGO_PKG_VERSION"))),
                          "Show version");
        parser.refer(&mut rom).add_option(&["-r", "--rom"], Parse, "Path to ROM file").required();
        parser.refer(&mut scale).add_option(&["-s", "--scale"], Parse, "Display scaling");
        parser.refer(&mut bootrom_path)
            .add_option(&["-b", "--bootrom"], Parse, "Path to boot ROM file");
        parser.parse_args_or_exit();
    }

    let cart = match cartridge::Cartridge::new(&rom) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to open cartridge: {} {}", rom.display(), e);
            process::exit(1)
        }
    };

    let bootrom_path = if bootrom_path == PathBuf::from("") {
        None
    } else {
        Some(&bootrom_path)
    };

    let bootrom = match bootrom::Bootrom::from_pathbuf(bootrom_path) {
        Ok(b) => b,
        Err(e) => {
            println!("Failed to load bootrom: {} {}",
                     bootrom_path.unwrap().display(),
                     e);
            process::exit(1)
        }
    };

    let scale = match scale {
        1 => Scale::X1,
        2 => Scale::X2,
        4 => Scale::X4,
        8 => Scale::X8,
        16 => Scale::X16,
        32 => Scale::X32,
        s => {
            println!("Invalid scale option: {}", s);
            println!("Possible scale options: 1, 2, 4, 8, 16, 32");
            process::exit(1)
        }
    };

    let mut window = Window::new("iogb",
                                 160,
                                 144,
                                 WindowOptions { scale: scale, ..WindowOptions::default() })
        .unwrap_or_else(|e| {
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
