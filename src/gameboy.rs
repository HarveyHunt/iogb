use cpu;
use cartridge;
use interconnect;

pub const CPU_HZ: u32 = 4194304;
pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;

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

    pub fn back_buffer(&self) -> &[u8; SCREEN_W * SCREEN_H] {
        &self.cpu.interconnect.gpu.buffer
    }
}
