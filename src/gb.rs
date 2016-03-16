use cpu;
use cartridge;
use interconnect;

#[derive(Debug)]
pub struct GameBoy {
    cpu: cpu::Cpu,
}

impl GameBoy {
    pub fn new(cart: cartridge::Cartridge) -> GameBoy {
        let ic = interconnect::Interconnect::new(cart);
        GameBoy { cpu: cpu::Cpu::new(ic) }
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.step();
        }
    }
}
