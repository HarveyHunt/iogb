use std::env::args;

mod gb;
mod cpu;
mod mmu;
mod cartridge;

fn main() {
    let args: Vec<_> = args().collect();

    if args.len() != 2 {
        panic!("usage: iogb <rom>");
    }

    let cart = cartridge::Cartridge::new(&args[1]);
    let gb = gb::GameBoy::new();

    println!("gb: {:?}", gb);
}
