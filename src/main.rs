mod gb;
mod cpu;
mod mmu;

fn main() {
    let gb = gb::GameBoy::new();

    println!("gb: {:?}", gb);
}
