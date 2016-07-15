use cpu;
use cartridge;
use interconnect;

#[derive(Debug)]
pub struct GameBoy {
    cpu: cpu::Cpu,
}

impl GameBoy {
    pub fn new(cart: cartridge::Cartridge, bootrom: Vec<u8>) -> GameBoy {
        let ic = interconnect::Interconnect::new(cart, bootrom);
        GameBoy { cpu: cpu::Cpu::new(ic) }
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.step();
        }
    }
}
