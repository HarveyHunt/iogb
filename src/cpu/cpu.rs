use mmu;

#[derive(Debug)]
pub struct Cpu {
    regs: Registers,
    mmu: mmu::Mmu,
}

pub enum Flags {
    C = 0x10,
    H = 0x20,
    N = 0x40,
    Z = 0x80,
}

#[derive(Debug)]
pub enum Regs {
    // 8 bit
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    // 16 bit
    PC,
    SP,
    // Pairs
    AF,
    BC,
    DE,
    HL,
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

impl Registers {
    pub fn readw(&self, reg: Regs) -> u16 {
        use self::Regs::*;
        match reg {
            PC => self.pc,
            SP => self.sp,
            AF => ((self.a as u16) << 8) | (self.f as u16),
            BC => ((self.b as u16) << 8) | (self.c as u16),
            DE => ((self.d as u16) << 8) | (self.e as u16),
            HL => ((self.h as u16) << 8) | (self.l as u16),
            _ => panic!("Unknown reg {:?}", reg),
        }
    }

    pub fn writew(&mut self, reg: Regs, val: u16) {
        use self::Regs::*;
        match reg {
            PC => self.pc = val,
            SP => self.sp = val,
            AF => {
                self.a = (val >> 8) as u8;
                self.f = val as u8
            } 
            BC => {
                self.b = (val >> 8) as u8;
                self.c = val as u8
            } 
            DE => {
                self.d = (val >> 8) as u8;
                self.e = val as u8
            } 
            HL => {
                self.h = (val >> 8) as u8;
                self.l = val as u8
            } 
            _ => panic!("Unknown reg {:?}", reg),
        }
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: Registers::default(),
            mmu: mmu::Mmu::new(),
        }
    }

    pub fn fetchb(&mut self) -> u8 {
        let val = self.mmu.readb(self.regs.pc);
        self.regs.pc += 1;
        val
    }

    pub fn fetchw(&mut self) -> u16 {
        let val = self.mmu.readw(self.regs.pc);
        self.regs.pc += 2;
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
