mod gb;
mod cpu;
mod mmu;
mod cartridge;

fn main() {
    let gb = gb::GameBoy::new();

    println!("gb: {:?}", gb);
}
