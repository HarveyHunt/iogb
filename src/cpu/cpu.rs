use mmu;

#[derive(Debug)]
pub struct Cpu {
    reg: Registers,
    mmu: mmu::Mmu,
}

pub enum Flags {
    C = 0x10,
    H = 0x20,
    N = 0x40,
    Z = 0x80,
}

#[derive(Debug, Default)]
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            reg: Registers::default(),
            mmu: mmu::Mmu::new(),
        }
    }

    pub fn fetchb(&mut self) -> u8 {
        let val = self.mmu.readb(self.reg.pc);
        self.reg.pc += 1;
        val
    }

    pub fn fetchw(&mut self) -> u16 {
        let val = self.mmu.readw(self.reg.pc);
        self.reg.pc += 2;
        val
    }

    // Decode and execute, returning the number of ticks that execution took.
    pub fn dexec(&mut self) -> u32 {
        let op = self.fetchb();
        match op {
            inv => {
                panic!("The instruction 0x{:x}@0x{:x} isn't implemented",
                       inv,
                       self.regs.pc)
            }
        }
    }
}
