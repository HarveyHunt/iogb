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

pub struct ImmediateB;
pub struct AddressW;

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

#[derive(Debug, Copy, Clone)]
pub enum IndirectAddr {
    SP,
    // Pairs
    AF,
    BC,
    DE,
    HL,
    HLP, // HL+
    HLM, // HL-
    ZeroPage,
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

trait ReadB {
    // TODO: Having &mut here is ugly
    fn readb(&self, cpu: &mut Cpu) -> u8;
}

trait WriteB {
    fn writeb(&self, cpu: &mut Cpu, val: u8);
}

trait ReadW {
    // TODO: Having &mut here is ugly
    fn readw(&self, cpu: &mut Cpu) -> u16;
}

// We can use the 16 bit contents of a register pair as a pointer into memory.
impl ReadB for IndirectAddr {
    fn readb(&self, cpu: &mut Cpu) -> u8 {
        let addr = cpu.iaddr(*self);
        cpu.mmu.readb(addr)
    }
}

impl ReadB for RegsB {
    fn readb(&self, cpu: &mut Cpu) -> u8 {
        cpu.regs.readb(*self)
    }
}

impl ReadB for ImmediateB {
    fn readb(&self, cpu: &mut Cpu) -> u8 {
        cpu.fetchb()
    }
}

impl WriteB for IndirectAddr {
    fn writeb(&self, cpu: &mut Cpu, val: u8) {
        let addr = cpu.iaddr(*self);
        cpu.mmu.writeb(addr, val);
    }
}

impl WriteB for RegsB {
    fn writeb(&self, cpu: &mut Cpu, val: u8) {
        cpu.regs.writeb(*self, val);
    }
}

impl ReadW for RegsW {
    fn readw(&self, cpu: &mut Cpu) -> u16 {
        cpu.regs.readw(*self)
    }
}

impl ReadW for AddressW {
    fn readw(&self, cpu: &mut Cpu) -> u16 {
        cpu.fetchw()
    }
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

    pub fn iaddr(&mut self, ia: IndirectAddr) -> u16 {
        use self::IndirectAddr::*;
        match ia {
            AF => self.regs.readw(self::RegsW::AF),
            BC => self.regs.readw(self::RegsW::BC),
            DE => self.regs.readw(self::RegsW::DE),
            SP => self.regs.readw(self::RegsW::SP),
            HL => self.regs.readw(self::RegsW::HL),
            HLP => {
                let val = self.regs.readw(self::RegsW::HL);
                self.regs.writew(self::RegsW::HL, val.wrapping_add(1));
                val
            }
            HLM => {
                let val = self.regs.readw(self::RegsW::HL);
                self.regs.writew(self::RegsW::HL, val.wrapping_sub(1));
                val
            }
            ZeroPage => 0xFF00 + self.fetchb() as u16,
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
            0x00 => self.nop(),
            0x01 => self.ldiw(BC),
            0x02 => self.ld(self::IndirectAddr::BC, A),
            0x03 => self.incw(BC),
            0x04 => self.inc(B),
            0x05 => self.dec(B),
            0x06 => self.ld(B, self::IndirectAddr::ZeroPage),
            0x09 => self.addw(BC),
            0x0A => self.ld(A, self::IndirectAddr::BC),
            0x0B => self.decw(BC),
            0x0C => self.inc(C),
            0x0D => self.dec(C),
            0x0E => self.ld(C, self::IndirectAddr::ZeroPage),
            0x11 => self.ldiw(DE),
            0x12 => self.ld(self::IndirectAddr::DE, A),
            0x13 => self.incw(DE),
            0x14 => self.inc(D),
            0x15 => self.dec(D),
            0x16 => self.ld(D, self::IndirectAddr::ZeroPage),
            0x19 => self.addw(DE),
            0x1A => self.ld(A, self::IndirectAddr::DE),
            0x1B => self.decw(DE),
            0x1C => self.inc(E),
            0x1D => self.dec(E),
            0x1E => self.ld(E, self::IndirectAddr::ZeroPage),
            0x21 => self.ldiw(HL),
            0x22 => self.ld(self::IndirectAddr::HLP, A),
            0x23 => self.incw(HL),
            0x24 => self.inc(H),
            0x25 => self.dec(H),
            0x26 => self.ld(H, self::IndirectAddr::ZeroPage),
            0x29 => self.addw(HL),
            0x2A => self.ld(A, self::IndirectAddr::HLP),
            0x2B => self.decw(HL),
            0x2C => self.inc(L),
            0x2D => self.dec(L),
            0x2E => self.ld(L, self::IndirectAddr::ZeroPage),
            0x2F => self.cpl(),
            0x31 => self.ldiw(SP),
            0x32 => self.ld(self::IndirectAddr::HLM, A),
            0x33 => self.incw(SP),
            0x34 => self.inc(self::IndirectAddr::HL),
            0x35 => self.dec(self::IndirectAddr::HL),
            0x36 => self.ld(self::IndirectAddr::HL, self::IndirectAddr::ZeroPage),
            0x37 => self.scf(),
            0x39 => self.addw(SP),
            0x3A => self.ld(A, self::IndirectAddr::HLM),
            0x3B => self.decw(SP),
            0x3C => self.inc(A),
            0x3D => self.dec(A),
            0x3E => self.ld(A, self::IndirectAddr::ZeroPage),
            0x3F => self.ccf(),
            0x40 => self.ld(B, B),
            0x41 => self.ld(B, C),
            0x42 => self.ld(B, D),
            0x43 => self.ld(B, E),
            0x44 => self.ld(B, H),
            0x45 => self.ld(B, L),
            0x46 => self.ld(B, self::IndirectAddr::HL),
            0x47 => self.ld(B, A),
            0x48 => self.ld(C, B),
            0x49 => self.ld(C, C),
            0x4A => self.ld(C, D),
            0x4B => self.ld(C, E),
            0x4C => self.ld(C, H),
            0x4D => self.ld(C, L),
            0x4E => self.ld(C, self::IndirectAddr::HL),
            0x4F => self.ld(C, A),
            0x50 => self.ld(D, B),
            0x51 => self.ld(D, C),
            0x52 => self.ld(D, D),
            0x53 => self.ld(D, E),
            0x54 => self.ld(D, H),
            0x55 => self.ld(D, L),
            0x56 => self.ld(D, self::IndirectAddr::HL),
            0x57 => self.ld(D, A),
            0x58 => self.ld(E, B),
            0x59 => self.ld(E, C),
            0x5A => self.ld(E, D),
            0x5B => self.ld(E, E),
            0x5C => self.ld(E, H),
            0x5D => self.ld(E, L),
            0x5E => self.ld(E, self::IndirectAddr::HL),
            0x5F => self.ld(E, A),
            0x60 => self.ld(H, B),
            0x61 => self.ld(H, C),
            0x62 => self.ld(H, D),
            0x63 => self.ld(H, E),
            0x64 => self.ld(H, H),
            0x65 => self.ld(H, L),
            0x66 => self.ld(H, self::IndirectAddr::HL),
            0x67 => self.ld(H, A),
            0x68 => self.ld(L, B),
            0x69 => self.ld(L, C),
            0x6A => self.ld(L, D),
            0x6B => self.ld(L, E),
            0x6C => self.ld(L, H),
            0x6D => self.ld(L, L),
            0x6E => self.ld(L, self::IndirectAddr::HL),
            0x6F => self.ld(L, A),
            0x70 => self.ld(self::IndirectAddr::HL, B),
            0x71 => self.ld(self::IndirectAddr::HL, C),
            0x72 => self.ld(self::IndirectAddr::HL, D),
            0x73 => self.ld(self::IndirectAddr::HL, E),
            0x74 => self.ld(self::IndirectAddr::HL, H),
            0x75 => self.ld(self::IndirectAddr::HL, L),
            0x77 => self.ld(self::IndirectAddr::HL, A),
            0x78 => self.ld(A, B),
            0x79 => self.ld(A, C),
            0x7A => self.ld(A, D),
            0x7B => self.ld(A, E),
            0x7C => self.ld(A, H),
            0x7D => self.ld(A, L),
            0x7E => self.ld(A, self::IndirectAddr::HL),
            0x7F => self.ld(A, A),
            0x80 => self.add(B),
            0x81 => self.add(C),
            0x82 => self.add(D),
            0x83 => self.add(E),
            0x84 => self.add(H),
            0x85 => self.add(L),
            0x86 => self.add(self::IndirectAddr::HL),
            0x87 => self.add(A),
            0x88 => self.adc(B),
            0x89 => self.adc(C),
            0x8A => self.adc(D),
            0x8B => self.adc(E),
            0x8C => self.adc(H),
            0x8D => self.adc(L),
            0x8E => self.adc(self::IndirectAddr::HL),
            0x8F => self.adc(A),
            0x90 => self.sub(B),
            0x91 => self.sub(C),
            0x92 => self.sub(D),
            0x93 => self.sub(E),
            0x94 => self.sub(H),
            0x95 => self.sub(L),
            0x96 => self.sub(self::IndirectAddr::HL),
            0x97 => self.sub(A),
            0x98 => self.sbc(B),
            0x99 => self.sbc(C),
            0x9A => self.sbc(D),
            0x9B => self.sbc(E),
            0x9C => self.sbc(H),
            0x9D => self.sbc(L),
            0x9E => self.sbc(self::IndirectAddr::HL),
            0x9F => self.sbc(A),
            0xA0 => self.and(B),
            0xA1 => self.and(C),
            0xA2 => self.and(D),
            0xA3 => self.and(E),
            0xA4 => self.and(H),
            0xA5 => self.and(L),
            0xA6 => self.and(self::IndirectAddr::HL),
            0xA7 => self.and(A),
            0xA8 => self.xor(B),
            0xA9 => self.xor(C),
            0xAA => self.xor(D),
            0xAB => self.xor(E),
            0xAC => self.xor(H),
            0xAD => self.xor(L),
            0xAE => self.xor(self::IndirectAddr::HL),
            0xAF => self.xor(A),
            0xB0 => self.or(B),
            0xB1 => self.or(C),
            0xB2 => self.or(D),
            0xB3 => self.or(E),
            0xB4 => self.or(H),
            0xB5 => self.or(L),
            0xB6 => self.or(self::IndirectAddr::HL),
            0xB7 => self.or(A),
            0xB8 => self.cp(B),
            0xB9 => self.cp(C),
            0xBA => self.cp(D),
            0xBB => self.cp(E),
            0xBC => self.cp(H),
            0xBD => self.cp(L),
            0xBE => self.cp(self::IndirectAddr::HL),
            0xBF => self.cp(A),
            0xC3 => self.jp(self::AddressW),
            0xC6 => self.add(self::ImmediateB),
            0xCE => self.adc(self::ImmediateB),
            0xD6 => self.sub(self::ImmediateB),
            0xDE => self.sbc(self::ImmediateB),
            0xE0 => self.ld(self::IndirectAddr::ZeroPage, A), // LDH
            0xEE => self.xor(self::ImmediateB),
            0xE6 => self.and(self::ImmediateB),
            0xE8 => self.addw_sp(),
            0xE9 => self.jp(HL),
            0xF0 => self.ld(A, self::IndirectAddr::ZeroPage), // LDH
            0xF6 => self.or(self::ImmediateB),
            0xF9 => self.ldw(SP, HL),
            0xFE => self.cp(self::ImmediateB),
            inv => {
                panic!("The instruction 0x{:x}@0x{:x} isn't implemented\n{:?}",
                       inv,
                       self.regs.pc,
                       self)
            }
        }
    }

    // INC ss
    // Z N H C
    // - - - - : 8
    fn incw(&mut self, reg: RegsW) -> u32 {
        let val = self.regs.readw(reg).wrapping_add(1);
        self.regs.writew(reg, val);
        8
    }

    // DEC ss
    // Z N H C
    // - - - - : 8
    fn decw(&mut self, reg: RegsW) -> u32 {
        let val = self.regs.readw(reg).wrapping_sub(1);
        self.regs.writew(reg, val);
        8
    }

    // INC r | (r)
    // Z N H C
    // Z 0 H - : 4 | 12
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
    // Z 1 H - : 4 | 8
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

    // LD d s | d (s) | (d) s | (d8) s | d (d8)
    // Z N H C
    // - - - - : 4 | 8 | 8 | 12 | 12
    fn ld<O: WriteB, I: ReadB>(&mut self, o: O, i: I) -> u32 {
        let v = i.readb(self);
        o.writeb(self, v);
        // TODO: Need to reflect how the timing is different for (r) and r.
        4
    }

    // LD dd nn
    // Z N H C
    // - - - - : 8
    fn ldw(&mut self, dd: RegsW, nn: RegsW) -> u32 {
        let v = self.regs.readw(nn);
        self.regs.writew(dd, v);
        8
    }

    // LD dd d16
    // Z N H C
    // - - - - : 12
    fn ldiw(&mut self, dd: RegsW) -> u32 {
        let v = self.fetchw();
        self.regs.writew(dd, v);
        12
    }

    // ADD HL ss
    // Z N H C
    // - 0 H C : 8
    fn addw(&mut self, ss: RegsW) -> u32 {
        use self::Flags::*;
        let hl = self.regs.readw(self::RegsW::HL);
        let val = self.regs.readw(ss);
        let out = val.wrapping_add(hl);

        self.set_flag(N, false);
        self.set_flag(H, (hl & 0x07FF) > 0x07FF + (val & 0x07FF));
        self.set_flag(C, hl > 0xFFFF - val);
        self.regs.writew(self::RegsW::HL, out);
        8
    }

    // ADD SP e
    // Z N H C
    // 0 0 H C : 16
    // TODO: Maybe we could treat r8 like ImmediateB - i.e. a pub struct...
    fn addw_sp(&mut self) -> u32 {
        use self::Flags::*;
        let sp = self.regs.readw(self::RegsW::SP);
        let val = self.fetchb() as i8 as i16 as u16;
        let out = sp.wrapping_add(val);

        self.set_flag(Z, false);
        self.set_flag(N, false);
        self.set_flag(H, (sp & 0x07FF) > 0x07FF + (val & 0x07FF));
        self.set_flag(C, sp > 0xFFFF - val);
        self.regs.writew(self::RegsW::SP, out);
        16
    }

    // ADD s | (s) | d8
    // Z N H C
    // Z 0 H C : 4 | 8 | 8
    fn add<I: ReadB>(&mut self, i: I) -> u32 {
        self.alu_addb(i, false)
    }

    // ADC s | (s) | d8
    // Z N H C
    // Z 0 H C : 4 | 8 | 8
    fn adc<I: ReadB>(&mut self, i: I) -> u32 {
        self.alu_addb(i, true)
    }

    fn alu_addb<I: ReadB>(&mut self, i: I, use_carry: bool) -> u32 {
        use self::Flags::*;
        let a = self.regs.readb(self::RegsB::A);
        let val = i.readb(self);
        let c = (self.check_flag(C) && use_carry) as u8;
        let out = val.wrapping_add(a).wrapping_add(c);

        self.set_flag(Z, out == 0);
        self.set_flag(N, false);
        self.set_flag(H, (a & 0xF) + c > 0xF - (val & 0xF));
        self.set_flag(C, a + c > 0xFF - val);
        self.regs.writeb(self::RegsB::A, out);
        // FIXME: This isn't correct... :-(
        4
    }

    // SUB s | (s) | d8
    // Z N H C
    // Z 1 H C : 4 | 8 | 8
    fn sub<I: ReadB>(&mut self, i: I) -> u32 {
        self.alu_subb(i, false)
    }

    // SBC s | (s) | d8
    // Z N H C
    // Z 1 H C : 4 | 8 | 8
    fn sbc<I: ReadB>(&mut self, i: I) -> u32 {
        self.alu_subb(i, true)
    }

    fn alu_subb<I: ReadB>(&mut self, i: I, use_carry: bool) -> u32 {
        use self::Flags::*;
        let a = self.regs.readb(self::RegsB::A);
        let val = i.readb(self);
        let c = (self.check_flag(C) && use_carry) as u8;
        let out = a.wrapping_sub(val).wrapping_sub(c);

        self.set_flag(Z, out == 0);
        self.set_flag(N, true);
        self.set_flag(H, a & 0xF < val & 0xF + c);
        self.set_flag(C, (a as u16) < (val as u16) + (c as u16));
        self.regs.writeb(self::RegsB::A, out);
        // TODO: Need to reflect how the timing is different for (r) and r.
        4
    }

    // CP s | (s) | d8
    // Z N H C
    // Z 1 H C : 4 | 8 | 8
    fn cp<I: ReadB>(&mut self, i: I) -> u32 {
        // This is kind of ugly, but I prefer having alu_subb handle all
        // changes to CPU state.
        let a = self.regs.readb(self::RegsB::A);
        let cycles = self.alu_subb(i, false);
        self.regs.writeb(self::RegsB::A, a);
        cycles
    }

    // OR s | (s) | d8
    // Z N H C
    // Z 0 0 0 : 4 | 8 | 8
    fn or<I: ReadB>(&mut self, i: I) -> u32 {
        use self::Flags::*;
        let mut v = i.readb(self);
        v |= self.regs.readb(self::RegsB::A);
        self.regs.writeb(self::RegsB::A, v);
        self.set_flag(Z, v == 0);
        self.set_flag(N, false);
        self.set_flag(H, false);
        self.set_flag(C, false);
        // TODO: Need to reflect how the timing is different for (r) and r.
        4
    }

    // XOR s | (s) | d8
    // Z N H C
    // Z 0 0 0 : 4 | 8 | 8
    fn xor<I: ReadB>(&mut self, i: I) -> u32 {
        use self::Flags::*;
        let mut v = i.readb(self);
        v ^= self.regs.readb(self::RegsB::A);
        self.regs.writeb(self::RegsB::A, v);
        self.set_flag(Z, v == 0);
        self.set_flag(N, false);
        self.set_flag(H, false);
        self.set_flag(C, false);
        // TODO: Need to reflect how the timing is different for (r) and r.
        4
    }

    // AND s | (s) | d8
    // Z N H C
    // Z 0 1 0 : 4 | 8 | 8
    fn and<I: ReadB>(&mut self, i: I) -> u32 {
        use self::Flags::*;
        let mut v = i.readb(self);
        v &= self.regs.readb(self::RegsB::A);
        self.regs.writeb(self::RegsB::A, v);
        self.set_flag(Z, v == 0);
        self.set_flag(N, false);
        self.set_flag(H, true);
        self.set_flag(C, false);
        // TODO: Need to reflect how the timing is different for (r) and r.
        4
    }

    // NOP
    // Z N H C
    // - - - - : 4
    fn nop(&mut self) -> u32 {
        4
    }

    // CPL
    // Z N H C
    // - 1 1 - : 4
    // Cyberathlete Professional League?
    fn cpl(&mut self) -> u32 {
        use self::Flags::*;
        let val = !self.regs.readb(self::RegsB::A);
        self.set_flag(N, true);
        self.set_flag(H, true);
        self.regs.writeb(self::RegsB::A, val);
        4
    }

    // CCF
    // Z N H C
    // - 0 0 C : 4
    fn ccf(&mut self) -> u32 {
        use self::Flags::*;
        let c = self.check_flag(C);
        self.set_flag(N, false);
        self.set_flag(H, false);
        self.set_flag(C, !c);
        4
    }

    // SCF
    // Z N H C
    // - 0 0 1 : 4
    fn scf(&mut self) -> u32 {
        use self::Flags::*;
        self.set_flag(N, false);
        self.set_flag(H, false);
        self.set_flag(C, true);
        4
    }

    // JP nn
    // Z N H C
    // - - - - : 4
    fn jp<I: ReadW>(&mut self, i: I) -> u32 {
        let addr = i.readw(self);
        self.regs.writew(self::RegsW::PC, addr);
        16
    }
}
