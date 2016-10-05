use cpu;
use cartridge;
use interconnect;

pub const CPU_HZ: u32 = 4194304;

#[derive(Debug)]
pub struct GameBoy {
    cpu: cpu::Cpu,
}

impl GameBoy {
    pub fn new(cart: cartridge::Cartridge, bootrom: Vec<u8>) -> GameBoy {
        let ic = interconnect::Interconnect::new(cart, bootrom);
        GameBoy { cpu: cpu::Cpu::new(ic) }
    }

    pub fn run(&mut self, timeslice: u32) -> u32 {
        let mut ticks = 0;
        loop {
            ticks += self.cpu.step();
            if ticks > timeslice {
                return ticks;
            }
        }
    }
}
