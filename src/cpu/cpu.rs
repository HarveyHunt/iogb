use mmu;
use cartridge;
use super::clk;

#[derive(Debug)]
pub struct Cpu {
    clk: clk::Clock,
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

trait WriteB {
    fn writeb(&self, cpu: &mut Cpu, val: u8);
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

impl WriteB for RegsW {
    fn writeb(&self, cpu: &mut Cpu, val: u8) {
        let addr = cpu.regs.readw(*self);
        cpu.mmu.writeb(addr, val);
    }
}

impl WriteB for RegsB {
    fn writeb(&self, cpu: &mut Cpu, val: u8) {
        cpu.regs.writeb(*self, val);
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

    pub fn writeb(&mut self, reg: RegsB, val: u8) {
        use self::RegsB::*;
        match reg {
            A => self.a = val,
            B => self.b = val,
            C => self.c = val,
            D => self.d = val,
            E => self.e = val,
            H => self.h = val,
            L => self.l = val,
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
    pub fn new(cart: cartridge::Cartridge) -> Cpu {
        Cpu {
            clk: clk::Clock::default(),
            regs: Registers::default(),
            mmu: mmu::Mmu::new(cart),
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

    pub fn set_flag(&mut self, flag: Flags, enable: bool) {
        let bit = flag as u8;
        self.regs.f = if enable {
            self.regs.f | bit
        } else {
            self.regs.f & !bit
        };
    }

    pub fn check_flag(&self, flag: Flags) -> bool {
        self.regs.f & (flag as u8) > 0
    }

    // Decode and execute, returning the number of ticks that execution took.
    pub fn dexec(&mut self) -> u32 {
        use self::RegsW::*;
        use self::RegsB::*;
        let op = self.fetchb();
        match op {
            0x02 => self.ld(BC, A),
            0x03 => self.incw(BC),
            0x04 => self.inc(B),
            0x05 => self.dec(B),
            0x0A => self.ld(A, BC),
            0x0C => self.inc(C),
            0x0D => self.dec(C),
            0x12 => self.ld(DE, A),
            0x13 => self.incw(DE),
            0x14 => self.inc(D),
            0x15 => self.dec(D),
            0x1A => self.ld(A, DE),
            0x1C => self.inc(E),
            0x1D => self.dec(E),
            0x24 => self.inc(H),
            0x25 => self.dec(H),
            0x2C => self.inc(L),
            0x2D => self.dec(L),
            // TODO: Maybe this should be clearer that we're using HL as a pointer...
            0x34 => self.inc(HL),
            0x35 => self.dec(HL),
            0x3C => self.inc(A),
            0x3D => self.dec(A),
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

    // INC r | (r)
    // Z N H C
    // Z 0 H - 4 (12)
    fn inc<A: ReadB + WriteB>(&mut self, addr: A) -> u32 {
        use self::Flags::*;
        let val = addr.readb(self).wrapping_add(1);
        self.set_flag(Z, val == 0);
        self.set_flag(N, false);
        self.set_flag(H, (val & 0xF) == 0x0);
        addr.writeb(self, val);
        // TODO: Need to reflect how the timing is different for (r) and r.
        4
    }

    // DEC r | (r)
    // Z N H C
    //
    // Z 1 H -
    fn dec<A: ReadB + WriteB>(&mut self, addr: A) -> u32 {
        use self::Flags::*;
        let val = addr.readb(self).wrapping_sub(1);
        self.set_flag(Z, val == 0);
        self.set_flag(N, true);
        self.set_flag(H, (val & 0xF) == 0xF);
        addr.writeb(self, val);
        // TODO: Need to reflect how the timing is different for (r) and r.
        4
    }

    // LD d s | d (s) | (d) s
    // Z N H C
    // - - - - 4 (12)
    fn ld<O: WriteB, I: ReadB>(&mut self, o: O, i: I) -> u32 {
        let v = i.readb(self);
        o.writeb(self, v);
        // TODO: Need to reflect how the timing is different for (r) and r.
        4
    }
}
