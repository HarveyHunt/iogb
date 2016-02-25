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

trait ReadB {
    // TODO: Having &mut here is ugly
    fn readb(&self, cpu: &mut Cpu) -> u8;
}

#[derive(Debug, Copy, Clone)]
pub enum RegsB {
    // 8 bit
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

// We can use the 16 bit contents of a register pair as a pointer into memory.
impl ReadB for RegsW {
    fn readb(&self, cpu: &mut Cpu) -> u8 {
        let addr = cpu.regs.readw(*self);
        cpu.mmu.readb(addr)
    }
}

impl ReadB for RegsB {
    fn readb(&self, cpu: &mut Cpu) -> u8 {
        cpu.regs.readb(*self)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RegsW {
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
    pub fn readb(&self, reg: RegsB) -> u8 {
        use self::RegsB::*;
        match reg {
            A => self.a,
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
        }
    }

    pub fn readw(&self, reg: RegsW) -> u16 {
        use self::RegsW::*;
        match reg {
            PC => self.pc,
            SP => self.sp,
            AF => ((self.a as u16) << 8) | (self.f as u16),
            BC => ((self.b as u16) << 8) | (self.c as u16),
            DE => ((self.d as u16) << 8) | (self.e as u16),
            HL => ((self.h as u16) << 8) | (self.l as u16),
        }
    }

    pub fn writew(&mut self, reg: RegsW, val: u16) {
        use self::RegsW::*;
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
        use self::RegsW::*;
        let op = self.fetchb();
        match op {
            0x03 => self.incw(BC),
            0x13 => self.incw(DE),
            0x23 => self.incw(HL),
            0x33 => self.incw(SP),
            0x0B => self.decw(BC),
            0x1B => self.decw(DE),
            0x2B => self.decw(HL),
            0x3B => self.decw(SP),
            inv => {
                panic!("The instruction 0x{:x}@0x{:x} isn't implemented",
                       inv,
                       self.regs.pc)
            }
        }
    }

    // INC ss
    // Z N H C
    // - - - - 8
    fn incw(&mut self, reg: RegsW) -> u32 {
        let val = self.regs.readw(reg).wrapping_add(1);
        self.regs.writew(reg, val);
        8
    }

    // DEC ss
    // Z N H C
    // - - - - 8
    fn decw(&mut self, reg: RegsW) -> u32 {
        let val = self.regs.readw(reg).wrapping_sub(1);
        self.regs.writew(reg, val);
        8
    }
}
