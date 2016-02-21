use cpu;

#[derive(Debug)]
pub struct GameBoy {
    cpu: cpu::Cpu,
}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy { cpu: cpu::Cpu::new() }
    }
}
