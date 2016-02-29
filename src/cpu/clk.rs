#[derive(Debug, Default)]
pub struct Clock {
    cycles: u32,
}

impl Clock {
    pub fn tick(&mut self) {
        self.cycles += 4;
    }
}
