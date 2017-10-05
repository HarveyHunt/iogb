use std::fmt;
use interconnect;
use super::clk;

#[derive(Debug)]
pub struct Cpu {
    clk: clk::Clock,
    regs: Registers,
    pub interconnect: interconnect::Interconnect,
}

#[derive(Debug)]
pub enum Flags {
    C = 0x10,
    H = 0x20,
    N = 0x40,
    Z = 0x80,
}

pub enum Condition {
    NZ,
    Z,
    NC,
    C,
}

#[derive(PartialEq)]
pub enum RotateDir {
    L,
    R,
}

impl Condition {
    // TODO: We don't want to take a reference to the whole CPU just to get to
    // the flags...
    pub fn test(&self, cpu: &Cpu) -> bool {
        use self::Condition::*;
        match *self {
            NZ => !cpu.check_flag(self::Flags::Z),
            Z => cpu.check_flag(self::Flags::Z),
            NC => !cpu.check_flag(self::Flags::C),
            C => cpu.check_flag(self::Flags::C),
        }
    }
}

pub struct ImmediateB;
pub struct ImmediateW;
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
    ZeroPageC,
    AddressW,
}

#[derive(Default)]
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

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Registers")
            .field("A", &format_args!("0x{:02x}", self.a))
            .field("B", &format_args!("0x{:02x}", self.b))
            .field("C", &format_args!("0x{:02x}", self.c))
            .field("D", &format_args!("0x{:02x}", self.d))
            .field("E", &format_args!("0x{:02x}", self.e))
            .field("F", &format_args!("0x{:02x}", self.f))
            .field("H", &format_args!("0x{:02x}", self.h))
            .field("L", &format_args!("0x{:02x}", self.l))
            .field("PC", &format_args!("0x{:04x}", self.pc))
            .field("SP", &format_args!("0x{:04x}", self.sp))
            .finish()
    }
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

impl ReadB for IndirectAddr {
    fn readb(&self, cpu: &mut Cpu) -> u8 {
        let addr = cpu.iaddr(*self);
        cpu.interconnect.readb(addr)
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
        cpu.interconnect.writeb(addr, val);
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
        if cfg!(debug_assertions) {
            print!("\t {:?}=0x{:02x}", reg, val);
        }
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
        if cfg!(debug_assertions) {
            print!("\t{:?}=0x{:04x}", reg, val);
        }
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
    pub fn new(interconnect: interconnect::Interconnect) -> Cpu {
        let mut cpu = Cpu {
            clk: clk::Clock::default(),
            regs: Registers::default(),
            interconnect: interconnect,
        };

        if !cpu.interconnect.brom.is_used() {
            cpu.fake_boot_regs();
        };

        cpu
    }

    fn fake_boot_regs(&mut self) {
        self.regs.writew(self::RegsW::AF, 0x01B0);
        self.regs.writew(self::RegsW::BC, 0x0013);
        self.regs.writew(self::RegsW::DE, 0x00D8);
        self.regs.writew(self::RegsW::HL, 0x014D);
        self.regs.writew(self::RegsW::SP, 0xFFFE);
    }

    fn print_stacktrace(&self) {
        let mut sp = self.regs.sp;
        let mut trace = "Stack:\n".to_owned();
        while sp > 0xFF80 {
            trace.push_str(&format!("0x{:04x}\n", self.interconnect.readw(sp)));
            sp -= 2;
        }
        println!("{}", trace);
    }

    fn crash(&self, cause: String) -> ! {
        let mut code: String = "Code:".to_owned();
        for pc in (self.regs.pc - 5)..(self.regs.pc + 4) {
            if (pc + 1) == self.regs.pc {
                code.push_str(&format!(" [0x{:02x}]", self.interconnect.readb(pc)));
            } else {
                code.push_str(&format!(" 0x{:02x}", self.interconnect.readb(pc)));
            }
        }
        println!("{}", code);

        if self.regs.sp < 0xFFFE {
            self.print_stacktrace();
        }

        println!("{:#?}", self);
        panic!("{}", cause);
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
            ZeroPageC => 0xFF00 + self.regs.readb(self::RegsB::C) as u16,
            AddressW => self.fetchw(),
        }
    }

    pub fn fetchb(&mut self) -> u8 {
        let val = self.interconnect.readb(self.regs.pc);
        self.regs.pc += 1;
        val
    }

    pub fn fetchw(&mut self) -> u16 {
        let val = self.interconnect.readw(self.regs.pc);
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

    fn handle_interrupts(&mut self) {
        let int;
        match self.interconnect.ic.get_interrupt() {
            None => return,
            Some(i) => int = i,
        }

        self.interconnect.ic.reset_interrupt(int);
        let pc = self.regs.readw(self::RegsW::PC);
        self.pushw(pc);
        self.regs.writew(self::RegsW::PC, int.get_addr());
        self.interconnect.ic.ime = false;
    }

    pub fn step(&mut self) -> u32 {
        if self.interconnect.ic.ime {
            self.handle_interrupts();
        }

        let ticks = self.dexec();
        self.clk.add_cycles(ticks);

        if cfg!(debug_assertions) {
            print!("\t F={:04b}", self.regs.f >> 4);
        }

        self.interconnect.step(ticks)
    }

    // Decode and execute, returning the number of ticks that execution took.
    pub fn dexec(&mut self) -> u32 {
        use self::RegsW::*;
        use self::RegsB::*;
        let op = self.fetchb();
        if cfg!(debug_assertions) {
            print!("\n0x{:02x}@0x{:04x}:", op, self.regs.pc - 1);
        }
        match op {
            0x00 => self.nop(),
            0x01 => self.ldw(BC, self::AddressW),
            0x02 => self.ld(self::IndirectAddr::BC, A),
            0x03 => self.incw(BC),
            0x04 => self.inc(B),
            0x05 => self.dec(B),
            0x06 => self.ld(B, self::ImmediateB),
            0x07 => self.rlca(),
            0x08 => self.ldw_nn_sp(),
            0x09 => self.addw(BC),
            0x10 => self.stop(),
            0x0A => self.ld(A, self::IndirectAddr::BC),
            0x0B => self.decw(BC),
            0x0C => self.inc(C),
            0x0D => self.dec(C),
            0x0E => self.ld(C, self::ImmediateB),
            0x0F => self.rrca(),
            0x11 => self.ldw(DE, self::AddressW),
            0x12 => self.ld(self::IndirectAddr::DE, A),
            0x13 => self.incw(DE),
            0x14 => self.inc(D),
            0x15 => self.dec(D),
            0x16 => self.ld(D, self::ImmediateB),
            0x17 => self.rla(),
            0x18 => self.jr(),
            0x19 => self.addw(DE),
            0x1A => self.ld(A, self::IndirectAddr::DE),
            0x1B => self.decw(DE),
            0x1C => self.inc(E),
            0x1D => self.dec(E),
            0x1E => self.ld(E, self::ImmediateB),
            0x1F => self.rra(),
            0x20 => self.jr_cond(self::Condition::NZ),
            0x21 => self.ldw(HL, self::AddressW),
            0x22 => self.ld(self::IndirectAddr::HLP, A),
            0x23 => self.incw(HL),
            0x24 => self.inc(H),
            0x25 => self.dec(H),
            0x26 => self.ld(H, self::ImmediateB),
            0x28 => self.jr_cond(self::Condition::Z),
            0x29 => self.addw(HL),
            0x2A => self.ld(A, self::IndirectAddr::HLP),
            0x2B => self.decw(HL),
            0x2C => self.inc(L),
            0x2D => self.dec(L),
            0x2E => self.ld(L, self::ImmediateB),
            0x2F => self.cpl(),
            0x30 => self.jr_cond(self::Condition::NC),
            0x31 => self.ldw(SP, self::AddressW),
            0x32 => self.ld(self::IndirectAddr::HLM, A),
            0x33 => self.incw(SP),
            0x34 => self.inc(self::IndirectAddr::HL),
            0x35 => self.dec(self::IndirectAddr::HL),
            0x36 => self.ld(self::IndirectAddr::HL, self::ImmediateB),
            0x37 => self.scf(),
            0x38 => self.jr_cond(self::Condition::C),
            0x39 => self.addw(SP),
            0x3A => self.ld(A, self::IndirectAddr::HLM),
            0x3B => self.decw(SP),
            0x3C => self.inc(A),
            0x3D => self.dec(A),
            0x3E => self.ld(A, self::ImmediateB),
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
            0xC0 => self.ret_cond(self::Condition::NZ),
            0xC1 => self.pop(BC),
            0xC2 => self.jp_cond(self::Condition::NZ),
            0xC3 => self.jp(self::AddressW),
            0xC4 => self.call_cond(self::Condition::NZ),
            0xC5 => self.push(BC),
            0xC6 => self.add(self::ImmediateB),
            0xC7 => self.rst(0x00),
            0xC8 => self.ret_cond(self::Condition::Z),
            0xC9 => self.ret(),
            0xCA => self.jp_cond(self::Condition::Z),
            0xCB => self.cb_dexec(),
            0xCC => self.call_cond(self::Condition::Z),
            0xCD => self.call(),
            0xCE => self.adc(self::ImmediateB),
            0xCF => self.rst(0x08),
            0xD0 => self.ret_cond(self::Condition::NC),
            0xD1 => self.pop(DE),
            0xD2 => self.jp_cond(self::Condition::NC),
            0xD4 => self.call_cond(self::Condition::NC),
            0xD5 => self.push(DE),
            0xD6 => self.sub(self::ImmediateB),
            0xD7 => self.rst(0x10),
            0xD8 => self.ret_cond(self::Condition::C),
            0xD9 => self.reti(),
            0xDA => self.jp_cond(self::Condition::C),
            0xDC => self.call_cond(self::Condition::C),
            0xDE => self.sbc(self::ImmediateB),
            0xDF => self.rst(0x18),
            0xE0 => self.ld(self::IndirectAddr::ZeroPage, A), // LDH
            0xE1 => self.pop(HL),
            0xE2 => self.ld(self::IndirectAddr::ZeroPageC, A), // LDH
            0xE5 => self.push(HL),
            0xE6 => self.and(self::ImmediateB),
            0xE7 => self.rst(0x20),
            0xE8 => self.addw_sp(),
            0xE9 => self.jp(HL),
            0xEA => self.ld(self::IndirectAddr::AddressW, A),
            0xEE => self.xor(self::ImmediateB),
            0xEF => self.rst(0x28),
            0xF0 => self.ld(A, self::IndirectAddr::ZeroPage), // LDH
            0xF1 => self.pop(AF),
            0xF2 => self.ld(A, self::IndirectAddr::ZeroPageC), // LDH
            0xF3 => self.di(),
            0xF5 => self.push(AF),
            0xF6 => self.or(self::ImmediateB),
            0xF7 => self.rst(0x30),
            0xF8 => self.ldw_hl_sp(),
            0xF9 => self.ldw(SP, HL),
            0xFA => self.ld(A, self::IndirectAddr::AddressW),
            0xFB => self.ei(),
            0xFE => self.cp(self::ImmediateB),
            0xFF => self.rst(0x38),
            inv => {
                self.crash(format!("The instruction 0x{:02x}@0x{:04x} isn't implemented",
                                   inv,
                                   self.regs.pc));
            }
        }
    }

    fn cb_dexec(&mut self) -> u32 {
        use self::RegsB::*;
        let op = self.fetchb();
        if cfg!(debug_assertions) {
            print!("\n0x{:02x}@0x{:04x}?", op, self.regs.pc - 1);
        }
        match op {
            0x00 => self.rlc(B),
            0x01 => self.rlc(C),
            0x02 => self.rlc(D),
            0x03 => self.rlc(E),
            0x04 => self.rlc(H),
            0x05 => self.rlc(L),
            0x06 => self.rlc(self::IndirectAddr::HL),
            0x07 => self.rlc(A),
            0x08 => self.rrc(B),
            0x09 => self.rrc(C),
            0x0A => self.rrc(D),
            0x0B => self.rrc(E),
            0x0C => self.rrc(H),
            0x0D => self.rrc(L),
            0x0E => self.rrc(self::IndirectAddr::HL),
            0x0F => self.rrc(A),
            0x10 => self.rl(B),
            0x11 => self.rl(C),
            0x12 => self.rl(D),
            0x13 => self.rl(E),
            0x14 => self.rl(H),
            0x15 => self.rl(L),
            0x16 => self.rl(self::IndirectAddr::HL),
            0x17 => self.rl(A),
            0x18 => self.rr(B),
            0x19 => self.rr(C),
            0x1A => self.rr(D),
            0x1B => self.rr(E),
            0x1C => self.rr(H),
            0x1D => self.rr(L),
            0x1E => self.rr(self::IndirectAddr::HL),
            0x1F => self.rr(A),
            0x20 => self.sla(B),
            0x21 => self.sla(C),
            0x22 => self.sla(D),
            0x23 => self.sla(E),
            0x24 => self.sla(H),
            0x25 => self.sla(L),
            0x26 => self.sla(self::IndirectAddr::HL),
            0x27 => self.sla(A),
            0x28 => self.sra(B),
            0x29 => self.sra(C),
            0x2A => self.sra(D),
            0x2B => self.sra(E),
            0x2C => self.sra(H),
            0x2D => self.sra(L),
            0x2E => self.sra(self::IndirectAddr::HL),
            0x2F => self.sra(A),
            0x30 => self.swap(B),
            0x31 => self.swap(C),
            0x32 => self.swap(D),
            0x33 => self.swap(E),
            0x34 => self.swap(H),
            0x35 => self.swap(L),
            0x36 => self.swap(self::IndirectAddr::HL),
            0x37 => self.swap(A),
            0x38 => self.srl(B),
            0x39 => self.srl(C),
            0x3A => self.srl(D),
            0x3B => self.srl(E),
            0x3C => self.srl(H),
            0x3D => self.srl(L),
            0x3E => self.srl(self::IndirectAddr::HL),
            0x3F => self.srl(A),
            0x40 => self.bit(0, B),
            0x41 => self.bit(0, C),
            0x42 => self.bit(0, D),
            0x43 => self.bit(0, E),
            0x44 => self.bit(0, H),
            0x45 => self.bit(0, L),
            0x46 => self.bit(0, self::IndirectAddr::HL),
            0x47 => self.bit(0, A),
            0x48 => self.bit(1, B),
            0x49 => self.bit(1, C),
            0x4A => self.bit(1, D),
            0x4B => self.bit(1, E),
            0x4C => self.bit(1, H),
            0x4D => self.bit(1, L),
            0x4E => self.bit(1, self::IndirectAddr::HL),
            0x4F => self.bit(1, A),
            0x50 => self.bit(2, B),
            0x51 => self.bit(2, C),
            0x52 => self.bit(2, D),
            0x53 => self.bit(2, E),
            0x54 => self.bit(2, H),
            0x55 => self.bit(2, L),
            0x56 => self.bit(2, self::IndirectAddr::HL),
            0x57 => self.bit(2, A),
            0x58 => self.bit(3, B),
            0x59 => self.bit(3, C),
            0x5A => self.bit(3, D),
            0x5B => self.bit(3, E),
            0x5C => self.bit(3, H),
            0x5D => self.bit(3, L),
            0x5E => self.bit(3, self::IndirectAddr::HL),
            0x5F => self.bit(3, A),
            0x60 => self.bit(4, B),
            0x61 => self.bit(4, C),
            0x62 => self.bit(4, D),
            0x63 => self.bit(4, E),
            0x64 => self.bit(4, H),
            0x65 => self.bit(4, L),
            0x66 => self.bit(4, self::IndirectAddr::HL),
            0x67 => self.bit(4, A),
            0x68 => self.bit(5, B),
            0x69 => self.bit(5, C),
            0x6A => self.bit(5, D),
            0x6B => self.bit(5, E),
            0x6C => self.bit(5, H),
            0x6D => self.bit(5, L),
            0x6E => self.bit(5, self::IndirectAddr::HL),
            0x6F => self.bit(5, A),
            0x70 => self.bit(6, B),
            0x71 => self.bit(6, C),
            0x72 => self.bit(6, D),
            0x73 => self.bit(6, E),
            0x74 => self.bit(6, H),
            0x75 => self.bit(6, L),
            0x76 => self.bit(6, self::IndirectAddr::HL),
            0x77 => self.bit(6, A),
            0x78 => self.bit(7, B),
            0x79 => self.bit(7, C),
            0x7A => self.bit(7, D),
            0x7B => self.bit(7, E),
            0x7C => self.bit(7, H),
            0x7D => self.bit(7, L),
            0x7E => self.bit(7, self::IndirectAddr::HL),
            0x7F => self.bit(7, A),
            0x80 => self.res(0, B),
            0x81 => self.res(0, C),
            0x82 => self.res(0, D),
            0x83 => self.res(0, E),
            0x84 => self.res(0, H),
            0x85 => self.res(0, L),
            0x86 => self.res(0, self::IndirectAddr::HL),
            0x87 => self.res(0, A),
            0x88 => self.res(1, B),
            0x89 => self.res(1, C),
            0x8A => self.res(1, D),
            0x8B => self.res(1, E),
            0x8C => self.res(1, H),
            0x8D => self.res(1, L),
            0x8E => self.res(1, self::IndirectAddr::HL),
            0x8F => self.res(1, A),
            0x90 => self.res(2, B),
            0x91 => self.res(2, C),
            0x92 => self.res(2, D),
            0x93 => self.res(2, E),
            0x94 => self.res(2, H),
            0x95 => self.res(2, L),
            0x96 => self.res(2, self::IndirectAddr::HL),
            0x97 => self.res(2, A),
            0x98 => self.res(3, B),
            0x99 => self.res(3, C),
            0x9A => self.res(3, D),
            0x9B => self.res(3, E),
            0x9C => self.res(3, H),
            0x9D => self.res(3, L),
            0x9E => self.res(3, self::IndirectAddr::HL),
            0x9F => self.res(3, A),
            0xA0 => self.res(4, B),
            0xA1 => self.res(4, C),
            0xA2 => self.res(4, D),
            0xA3 => self.res(4, E),
            0xA4 => self.res(4, H),
            0xA5 => self.res(4, L),
            0xA6 => self.res(4, self::IndirectAddr::HL),
            0xA7 => self.res(4, A),
            0xA8 => self.res(5, B),
            0xA9 => self.res(5, C),
            0xAA => self.res(5, D),
            0xAB => self.res(5, E),
            0xAC => self.res(5, H),
            0xAD => self.res(5, L),
            0xAE => self.res(5, self::IndirectAddr::HL),
            0xAF => self.res(5, A),
            0xB0 => self.res(6, B),
            0xB1 => self.res(6, C),
            0xB2 => self.res(6, D),
            0xB3 => self.res(6, E),
            0xB4 => self.res(6, H),
            0xB5 => self.res(6, L),
            0xB6 => self.res(6, self::IndirectAddr::HL),
            0xB7 => self.res(6, A),
            0xB8 => self.res(7, B),
            0xB9 => self.res(7, C),
            0xBA => self.res(7, D),
            0xBB => self.res(7, E),
            0xBC => self.res(7, H),
            0xBD => self.res(7, L),
            0xBE => self.res(7, self::IndirectAddr::HL),
            0xBF => self.res(7, A),
            0xC0 => self.set(0, B),
            0xC1 => self.set(0, C),
            0xC2 => self.set(0, D),
            0xC3 => self.set(0, E),
            0xC4 => self.set(0, H),
            0xC5 => self.set(0, L),
            0xC6 => self.set(0, self::IndirectAddr::HL),
            0xC7 => self.set(0, A),
            0xC8 => self.set(1, B),
            0xC9 => self.set(1, C),
            0xCA => self.set(1, D),
            0xCB => self.set(1, E),
            0xCC => self.set(1, H),
            0xCD => self.set(1, L),
            0xCE => self.set(1, self::IndirectAddr::HL),
            0xCF => self.set(1, A),
            0xD0 => self.set(2, B),
            0xD1 => self.set(2, C),
            0xD2 => self.set(2, D),
            0xD3 => self.set(2, E),
            0xD4 => self.set(2, H),
            0xD5 => self.set(2, L),
            0xD6 => self.set(2, self::IndirectAddr::HL),
            0xD7 => self.set(2, A),
            0xD8 => self.set(3, B),
            0xD9 => self.set(3, C),
            0xDA => self.set(3, D),
            0xDB => self.set(3, E),
            0xDC => self.set(3, H),
            0xDD => self.set(3, L),
            0xDE => self.set(3, self::IndirectAddr::HL),
            0xDF => self.set(3, A),
            0xE0 => self.set(4, B),
            0xE1 => self.set(4, C),
            0xE2 => self.set(4, D),
            0xE3 => self.set(4, E),
            0xE4 => self.set(4, H),
            0xE5 => self.set(4, L),
            0xE6 => self.set(4, self::IndirectAddr::HL),
            0xE7 => self.set(4, A),
            0xE8 => self.set(5, B),
            0xE9 => self.set(5, C),
            0xEA => self.set(5, D),
            0xEB => self.set(5, E),
            0xEC => self.set(5, H),
            0xED => self.set(5, L),
            0xEE => self.set(5, self::IndirectAddr::HL),
            0xEF => self.set(5, A),
            0xF0 => self.set(6, B),
            0xF1 => self.set(6, C),
            0xF2 => self.set(6, D),
            0xF3 => self.set(6, E),
            0xF4 => self.set(6, H),
            0xF5 => self.set(6, L),
            0xF6 => self.set(6, self::IndirectAddr::HL),
            0xF7 => self.set(6, A),
            0xF8 => self.set(7, B),
            0xF9 => self.set(7, C),
            0xFA => self.set(7, D),
            0xFB => self.set(7, E),
            0xFC => self.set(7, H),
            0xFD => self.set(7, L),
            0xFE => self.set(7, self::IndirectAddr::HL),
            0xFF => self.set(7, A),
            inv => {
                self.crash(format!("The CB instruction 0x{:02x}@0x{:04x} isn't implemented",
                                   inv,
                                   self.regs.pc));
            }
        }
    }

    // STOP
    // Z N H C
    // - - - - : 4
    fn stop(&mut self) -> ! {
        self.crash("STOP".to_owned());
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

    // LD dd nn | dd d16
    // Z N H C
    // - - - - : 8 | 12
    fn ldw<I: ReadW>(&mut self, dd: RegsW, i: I) -> u32 {
        let v = i.readw(self);
        self.regs.writew(dd, v);
        8
    }

    // LD (nn) SP
    // Z N H C
    // - - - - : 20
    fn ldw_nn_sp(&mut self) -> u32 {
        let addr = self.fetchw();
        let sp = self.regs.readw(self::RegsW::SP);
        self.interconnect.writew(addr, sp);
        20
    }

    // LD HL SP+r8
    // Z N H C
    // 0 0 H C : 12
    fn ldw_hl_sp(&mut self) -> u32 {
        use self::Flags::*;
        let offset = self.fetchb() as i8 as u16;
        let sp = self.regs.readw(self::RegsW::SP);
        self.regs.writew(self::RegsW::HL, sp.wrapping_add(offset));

        self.set_flag(Z, false);
        self.set_flag(N, false);
        self.set_flag(H, (offset & 0x000F) + (sp & 0x000F) > 0x000F);
        self.set_flag(C, (offset & 0x00FF) + (sp & 0x00FF) > 0x00FF);
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
        self.set_flag(H, a & 0xF < (val & 0xF) + c);
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

    // JP cc nn
    // Z N H C
    // - - - - : 16/12
    fn jp_cond(&mut self, c: Condition) -> u32 {
        let addr = self.fetchw();
        if !c.test(self) {
            return 12;
        }
        self.regs.writew(self::RegsW::PC, addr);
        16
    }

    // JR e
    // Z N H C
    // - - - - : 12
    fn jr(&mut self) -> u32 {
        let mut addr = self.fetchb() as i8 as i16;
        addr += self.regs.readw(self::RegsW::PC) as i16;
        self.regs.writew(self::RegsW::PC, addr as u16);
        12
    }

    // JR cc e
    // Z N H C
    // - - - - : 12/8
    fn jr_cond(&mut self, c: Condition) -> u32 {
        let mut addr = self.fetchb() as i8 as i16;
        if !c.test(self) {
            return 8;
        }

        addr += self.regs.readw(self::RegsW::PC) as i16;
        self.regs.writew(self::RegsW::PC, addr as u16);
        12
    }

    // PUSH qq
    // Z N H C
    // - - - - 16
    fn push(&mut self, reg: self::RegsW) -> u32 {
        let addr = self.regs.readw(reg);
        self.pushw(addr);
        16
    }

    fn pushw(&mut self, val: u16) {
        let sp = self.regs.readw(self::RegsW::SP).wrapping_sub(2);
        self.interconnect.writew(sp, val);
        self.regs.writew(self::RegsW::SP, sp);
    }

    // POP qq
    // Z N H C
    // - - - - 12
    // FIXME: POP AF affects _all_ flags, but I don't yet know how...
    fn pop(&mut self, reg: self::RegsW) -> u32 {
        let val = self.popw();
        self.regs.writew(reg, val);
        12
    }

    fn popw(&mut self) -> u16 {
        let mut sp = self.regs.readw(self::RegsW::SP);
        let mut val = self.interconnect.readb(sp) as u16;
        sp = sp.wrapping_add(1);
        val |= (self.interconnect.readb(sp) as u16) << 8;
        self.regs.writew(self::RegsW::SP, sp.wrapping_add(1));
        val
    }

    // CALL nn
    // Z N H C
    // - - - - 24
    fn call(&mut self) -> u32 {
        let new_pc = self.fetchw();
        let pc = self.regs.readw(self::RegsW::PC);

        self.pushw(pc);
        self.regs.writew(self::RegsW::PC, new_pc);
        24
    }

    // CALL cc nn
    // Z N H C
    // - - - - 24/12
    fn call_cond(&mut self, cond: Condition) -> u32 {
        let new_pc = self.fetchw();

        if !cond.test(self) {
            return 12;
        }

        let pc = self.regs.readw(self::RegsW::PC);
        self.pushw(pc);
        self.regs.writew(self::RegsW::PC, new_pc);
        24
    }

    fn do_ret(&mut self) -> u32 {
        let pc = self.popw();
        self.regs.writew(self::RegsW::PC, pc);
        16
    }

    // RETI
    // Z N H C
    // - - - - 16
    fn reti(&mut self) -> u32 {
        self.interconnect.ic.ime = true;
        self.do_ret()
    }

    // RET
    // Z N H C
    // - - - - 16
    fn ret(&mut self) -> u32 {
        self.do_ret()
    }

    // RET cc
    // Z N H C
    // - - - - 20/8
    fn ret_cond(&mut self, cond: Condition) -> u32 {
        if !cond.test(self) {
            return 8;
        }
        self.do_ret() + 4
    }

    // RST t
    // Z N H C
    // - - - - 16
    fn rst(&mut self, addr: u8) -> u32 {
        let pc = self.regs.readw(self::RegsW::PC);
        self.pushw(pc);
        self.regs.writew(self::RegsW::PC, addr as u16);
        16
    }

    // RLCA
    // Z N H C
    // 0 0 0 C 4
    fn rlca(&mut self) -> u32 {
        let mut a = self.regs.readb(self::RegsB::A);
        a = self.alu_rxx(self::RotateDir::L, false, false, a);
        self.regs.writeb(self::RegsB::A, a);
        4
    }

    // RLA
    // Z N H C
    // 0 0 0 C 4
    fn rla(&mut self) -> u32 {
        let mut a = self.regs.readb(self::RegsB::A);
        a = self.alu_rxx(self::RotateDir::L, true, false, a);
        self.regs.writeb(self::RegsB::A, a);
        4
    }

    // RRCA
    // Z N H C
    // 0 0 0 C 4
    fn rrca(&mut self) -> u32 {
        let mut a = self.regs.readb(self::RegsB::A);
        a = self.alu_rxx(self::RotateDir::R, false, false, a);
        self.regs.writeb(self::RegsB::A, a);
        4
    }

    // RRA
    // Z N H C
    // 0 0 0 C 4
    fn rra(&mut self) -> u32 {
        let mut a = self.regs.readb(self::RegsB::A);
        a = self.alu_rxx(self::RotateDir::R, true, false, a);
        self.regs.writeb(self::RegsB::A, a);
        4
    }

    fn rl<A: ReadB + WriteB>(&mut self, addr: A) -> u32 {
        let mut v = addr.readb(self);
        v = self.alu_rxx(self::RotateDir::L, true, true, v);
        addr.writeb(self, v);
        8
    }

    fn rlc<A: ReadB + WriteB>(&mut self, addr: A) -> u32 {
        let mut v = addr.readb(self);
        v = self.alu_rxx(self::RotateDir::L, false, true, v);
        addr.writeb(self, v);
        8
    }

    fn rr<A: ReadB + WriteB>(&mut self, addr: A) -> u32 {
        let mut v = addr.readb(self);
        v = self.alu_rxx(self::RotateDir::R, true, true, v);
        addr.writeb(self, v);
        8
    }

    fn rrc<A: ReadB + WriteB>(&mut self, addr: A) -> u32 {
        let mut v = addr.readb(self);
        v = self.alu_rxx(self::RotateDir::R, false, true, v);
        addr.writeb(self, v);
        8
    }

    fn alu_rxx(&mut self, dir: RotateDir, include_carry: bool, set_z: bool, val: u8) -> u8 {
        use self::Flags::*;
        let cout: bool;
        let out: u8;

        if dir == self::RotateDir::L {
            cout = (val & 0x80) != 0;
            if include_carry {
                out = val << 1 | self.check_flag(C) as u8;
            } else {
                out = val.rotate_left(1);
            }
        } else {
            cout = (val & 0x01) != 0;
            if include_carry {
                out = val >> 1 | (self.check_flag(C) as u8) << 7;
            } else {
                out = val.rotate_right(1);
            }
        }

        self.set_flag(Z, out == 0 && set_z);
        self.set_flag(N, false);
        self.set_flag(H, false);
        self.set_flag(C, cout);
        out
    }

    fn alu_sxx(&mut self, dir: RotateDir, preserve_msb: bool, val: u8) -> u8 {
        use self::Flags::*;
        let cout: bool;

        let out = if dir == self::RotateDir::L {
            cout = (val & 0x80) != 0;
            val << 1
        } else {
            let msb = (preserve_msb as u8 & (val & 0x80)) << 7;
            cout = (val & 0x01) != 0;
            val >> 1 | msb
        };

        self.set_flag(Z, out == 0);
        self.set_flag(N, false);
        self.set_flag(H, false);
        self.set_flag(C, cout);
        out
    }

    // SLA r | (hl)
    // Z N H C
    // Z 0 0 C : 8 | 12
    fn sla<A: ReadB + WriteB>(&mut self, addr: A) -> u32 {
        let mut v = addr.readb(self);
        v = self.alu_sxx(self::RotateDir::L, false, v);
        addr.writeb(self, v);
        8
    }

    // SRA r | (hl)
    // Z N H C
    // Z 0 0 C : 8 | 12
    fn sra<A: ReadB + WriteB>(&mut self, addr: A) -> u32 {
        let mut v = addr.readb(self);
        v = self.alu_sxx(self::RotateDir::R, true, v);
        addr.writeb(self, v);
        8
    }

    // SRL r | (hl)
    // Z N H C
    // Z 0 0 C : 8 | 12
    fn srl<A: ReadB + WriteB>(&mut self, addr: A) -> u32 {
        let mut v = addr.readb(self);
        v = self.alu_sxx(self::RotateDir::R, false, v);
        addr.writeb(self, v);
        8
    }

    // EI
    // Z N H C
    // - - - - 4
    fn ei(&mut self) -> u32 {
        self.interconnect.ic.enable_all_interrupts(1);
        4
    }

    // DI
    // Z N H C
    // - - - - 4
    fn di(&mut self) -> u32 {
        self.interconnect.ic.enable_all_interrupts(0);
        4
    }

    // BIT b r | b (hl)
    // Z N H C
    // Z 0 1 - 8 | 16
    fn bit<I: ReadB>(&mut self, b: u8, i: I) -> u32 {
        use self::Flags::*;
        let z = (i.readb(self) & (1 << b)) == 0;
        self.set_flag(Z, z);
        self.set_flag(N, false);
        self.set_flag(H, true);
        // TODO: Return correct value...
        8
    }

    // SET b r | b (hl)
    // Z N H C
    // - - - - 8 | 16
    fn set<A: ReadB + WriteB>(&mut self, bit: u8, addr: A) -> u32 {
        let val = addr.readb(self) | 1 << bit;
        addr.writeb(self, val | 1 << bit);
        // TODO: Return correct value...
        8
    }

    // RES b r | b (hl)
    // Z N H C
    // - - - - 8 | 16
    fn res<A: ReadB + WriteB>(&mut self, bit: u8, addr: A) -> u32 {
        let val = addr.readb(self) | 1 << bit;
        addr.writeb(self, val & 0 << bit);
        // TODO: Return correct value...
        8
    }

    // SWAP r | (hl)
    // Z N H C
    // Z - - - 8 | 16
    fn swap<A: ReadB + WriteB>(&mut self, addr: A) -> u32 {
        use self::Flags::*;
        let v = addr.readb(self);
        let out = (v >> 4) | (v << 4);
        self.set_flag(Z, out == 0);
        self.set_flag(N, false);
        self.set_flag(H, false);
        self.set_flag(C, false);
        // TODO: Return correct value...
        8
    }
}
