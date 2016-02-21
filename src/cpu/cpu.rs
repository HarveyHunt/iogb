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
}
