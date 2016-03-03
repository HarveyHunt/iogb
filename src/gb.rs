use cpu;
use cartridge;

#[derive(Debug)]
pub struct GameBoy {
    cpu: cpu::Cpu,
}

impl GameBoy {
    pub fn new(cart: cartridge::Cartridge) -> GameBoy {
        GameBoy { cpu: cpu::Cpu::new(cart) }
    }
}
