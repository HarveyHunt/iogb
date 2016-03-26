#[derive(Debug, Default)]
pub struct Clock {
    cycles: u32,
}

impl Clock {
    pub fn tick(&mut self) {
        self.cycles += 4;
    }

    pub fn add_cycles(&mut self, cycles: u32) {
        self.cycles += cycles;
    }
}
