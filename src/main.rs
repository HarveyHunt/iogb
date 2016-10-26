#![deny(trivial_casts, trivial_numeric_casts)]
extern crate argparse;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate glium;
extern crate time;
extern crate cgmath;

use std::path::PathBuf;
use std::process;
use time::{SteadyTime, Duration};
use argparse::{ArgumentParser, Parse, Print};
use glium::{Surface, DisplayBuild};

mod gameboy;
mod renderer;
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

    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(160, 144)
        .with_title("iogb - ".to_owned() + &cart.title)
        .build_glium()
        .unwrap();

    let mut gb = gameboy::GameBoy::new(cart, bootrom);
    let mut renderer = renderer::Renderer::new(&display);
    let mut ticks = 0;
    let mut delta: Duration;
    let mut last_time = SteadyTime::now();

    loop {
        let now = SteadyTime::now();
        delta = now - last_time;
        last_time = now;

        for event in display.poll_events() {
            use glium::glutin::Event;

            match event {
                Event::Closed => return,
                Event::Resized(width, height) => renderer.resize(width, height),
                _ => {}
            }
        }

        // TODO: Receive VSYNC event so we can regenerate the texture
        // from the GPU's back buffer.
        gb.run((delta * gameboy::CPU_HZ as i32).num_seconds() as u32);

        renderer.update_texture(gb.back_buffer());

        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 1.0);
        renderer.render(&mut target);
        target.finish().unwrap();
    }
}
