use std::fmt;

use super::brom::{BROM_SZ, BOOTROM};
use cartridge;

const WRAM_SZ: usize = 0x8000;
const ZRAM_SZ: usize = 0x7F;

pub struct Interconnect {
    brom: [u8; BROM_SZ], // 0x0000 -> 0x00FF
    wram: [u8; WRAM_SZ], // 0xC000 -> 0xDFFF, shadowed @ 0xE000 -> 0xFDFF
    zram: [u8; ZRAM_SZ], // 0xFF80 -> 0xFFFF
    cart: cartridge::Cartridge,
    boot_mode: bool, // Map brom into bottom of memory?
}

impl Interconnect {
    pub fn new(cart: cartridge::Cartridge) -> Interconnect {
        Interconnect {
            brom: BOOTROM,
            wram: [0; WRAM_SZ],
            zram: [0; ZRAM_SZ],
            cart: cart,
            boot_mode: true,
        }
    }

    pub fn readb(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x00FF => {
                if self.boot_mode {
                    self.brom[addr as usize]
                } else {
                    self.cart.read_rom(addr)
                }
            }
            0x0100...0x7FFF => self.cart.read_rom(addr), 
            0x8000...0x9FFF => 0, // GPU
            0xA000...0xBFFF => self.cart.read_ram(addr),
            // TODO: 0xD000 -> 0xDFFF is banked on CGB
            0xC000...0xDFFF => self.wram[addr as usize & 0x1FFF],
            0xE000...0xFDFF => self.wram[addr as usize & 0x1FFF],
            0xFE00...0xFE9F => 0, // OAM
            0xFF00...0xFF7F => 0, //MMIO
            0xFF80...0xFFFE => self.zram[addr as usize & 0x7F],
            0xFFFF => 0, //FIXME: Need a way of accessing the CPU's IC...
            _ => panic!("Can't read 0x{:x}", addr),
        }
    }

    pub fn writeb(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x7FFF => self.cart.write_rom(addr, val),
            0x8000...0x9FFF => {} // GPU
            0xA000...0xBFFF => self.cart.write_ram(addr, val),
            // TODO: 0xD000 -> 0xDFFF is banked on CGB
            0xC000...0xDFFF => self.wram[addr as usize & 0x1FFF] = val,
            0xE000...0xFDFF => self.wram[addr as usize & 0x1FFF] = val,
            0xFE00...0xFE9F => {} // OAM
            0xFF00...0xFF7F => {} //MMIO
            0xFF80...0xFFFE => self.zram[addr as usize & 0x7F] = val,
            0xFFFF => {} //FIXME: Need a way of accessing the CPU's IC...
            _ => panic!("Can't write to 0x{:x}", addr),
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

impl fmt::Debug for Interconnect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "interconnect: boot_mode: {}", self.boot_mode)
    }
}
