use std::fmt;

const BIOS_SZ: usize = 0x100;
const WRAM_SZ: usize = 0x8000;
const ZRAM_SZ: usize = 0x7F;

pub struct Mmu {
    bios: [u8; BIOS_SZ], // 0x0000 -> 0x00FF
    wram: [u8; WRAM_SZ], // 0xC000 -> 0xDFFF, shadowed @ 0xE000 -> 0xFDFF
    zram: [u8; ZRAM_SZ], // 0xFF80 -> 0xFFFF
}

impl Mmu {
    pub fn new() -> Mmu {
        Mmu {
            bios: [0; BIOS_SZ],
            wram: [0; WRAM_SZ],
            zram: [0; ZRAM_SZ],
        }
    }

    pub fn readb(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000...0x7FFF => 0, //0x0000 -> 0x3FFF MBC0, 0x4000 -> 0x7FFF MBCx
            0x8000...0x9FFF => 0, // GPU
            0xA000...0xBFFF => 0, //Cartridge RAM
            // TODO: 0xD000 -> 0xDFFF is banked on CGB
            0xC000...0xDFFF => self.wram[addr as usize & 0x1FFF],
            0xE000...0xFDFF => self.wram[addr as usize & 0x1FFF],
            0xFE00...0xFE9F => 0, // OAM
            0xFF00...0xFF7F => 0, //MMIO
            0xFF80...0xFFFE => self.zram[addr as usize & 0x7F],
            _ => panic!("Can't read {:x}", addr),
        }
    }

    pub fn writeb(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x7FFF => {} //0x0000 -> 0x3FFF MBC0, 0x4000 -> 0x7FFF MBCx
            0x8000...0x9FFF => {} // GPU
            0xA000...0xBFFF => {} //Cartridge RAM
            // TODO: 0xD000 -> 0xDFFF is banked on CGB
            0xC000...0xDFFF => self.wram[addr as usize & 0x1FFF] = val,
            0xE000...0xFDFF => self.wram[addr as usize & 0x1FFF] = val,
            0xFE00...0xFE9F => {} // OAM
            0xFF00...0xFF7F => {} //MMIO
            0xFF80...0xFFFE => self.zram[addr as usize & 0x7F] = val,
            _ => panic!("Can't write to {:x}", addr),
        }
    }

    pub fn readw(&mut self, addr: u16) -> u16 {
        (self.readb(addr) as u16) | ((self.readb(addr + 1) as u16) << 8)
    }

    pub fn writew(&mut self, addr: u16, val: u16) {
        self.writeb(addr, (val & 0xFF) as u8);
        self.writeb(addr + 1, (val >> 8) as u8);
    }
}

impl fmt::Debug for Mmu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "mmu debug stub")
    }
}
