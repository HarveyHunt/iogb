use std::fmt;

use super::brom::{BROM_SZ, BOOTROM};
use interrupt;
use cartridge;
use timer;
use gpu;

const WRAM_SZ: usize = 0x8000;
const ZRAM_SZ: usize = 0x7F;

pub struct Interconnect {
    brom: [u8; BROM_SZ], // 0x0000 -> 0x00FF
    wram: [u8; WRAM_SZ], // 0xC000 -> 0xDFFF, shadowed @ 0xE000 -> 0xFDFF
    zram: [u8; ZRAM_SZ], // 0xFF80 -> 0xFFFF
    cart: cartridge::Cartridge,
    boot_mode: bool, // Map brom into bottom of memory?
    // TODO: Make this private and implement wrapper functions
    pub ic: interrupt::InterruptController,
    pub timer: timer::Timer,
    gpu: gpu::Gpu,
}

impl Interconnect {
    pub fn new(cart: cartridge::Cartridge) -> Interconnect {
        Interconnect {
            brom: BOOTROM,
            wram: [0; WRAM_SZ],
            zram: [0; ZRAM_SZ],
            cart: cart,
            boot_mode: true,
            ic: interrupt::InterruptController::new(),
            timer: timer::Timer::new(),
            gpu: gpu::Gpu::new(),
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
            0x8000...0x9FFF => self.gpu.read_vram(addr),
            0xA000...0xBFFF => self.cart.read_ram(addr),
            // TODO: 0xD000 -> 0xDFFF is banked on CGB
            0xC000...0xDFFF => self.wram[addr as usize & 0x1FFF],
            0xE000...0xFDFF => self.wram[addr as usize & 0x1FFF],
            0xFE00...0xFE9F => self.gpu.read_oam(addr & 0x9F),
            0xFF00...0xFF03 => 0, //MMIO
            0xFF04 => self.timer.get_div(), 
            0xFF05 => self.timer.get_tima(), 
            0xFF06 => self.timer.get_tma(), 
            0xFF07 => self.timer.get_tac(), 
            0xFF08...0xFF0E => 0, //MMIO
            0xFF0F => self.ic.iflag,
            0xFF10...0xFF3F => 0, //MMIO
            0xFF40 => self.gpu.read_lcdc_reg(),
            0xFF41 => self.gpu.read_stat(),
            0xFF42 => self.gpu.read_scy(),
            0xFF43 => self.gpu.read_scx(),
            0xFF44 => self.gpu.read_ly(), 
            0xFF45 => self.gpu.read_lyc(), 
            0xFF46 => 0, //MMIO
            0xFF47 => self.gpu.read_bgp(), //MMIO
            0xFF48 => self.gpu.read_obp0(), //MMIO
            0xFF49 => self.gpu.read_obp1(), //MMIO
            0xFF4A => self.gpu.read_wy(),
            0xFF4B => self.gpu.read_wx(),
            0xFF4C...0xFF4F => 0, //MMIO
            0xFF80...0xFFFE => self.zram[addr as usize & 0x7F],
            0xFFFF => self.ic.ie, 
            _ => panic!("Can't read 0x{:04x}", addr),
        }
    }

    pub fn writeb(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x7FFF => self.cart.write_rom(addr, val),
            0x8000...0x9FFF => self.gpu.write_vram(addr, val),
            0xA000...0xBFFF => self.cart.write_ram(addr, val),
            // TODO: 0xD000 -> 0xDFFF is banked on CGB
            0xC000...0xDFFF => self.wram[addr as usize & 0x1FFF] = val,
            0xE000...0xFDFF => self.wram[addr as usize & 0x1FFF] = val,
            0xFE00...0xFE9F => self.gpu.write_oam(addr & 0x9F, val),
            0xFF00...0xFF03 => {} //MMIO
            0xFF04 => self.timer.set_div(val), 
            0xFF05 => self.timer.set_tima(val), 
            0xFF06 => self.timer.set_tma(val), 
            0xFF07 => self.timer.set_tac(val), 
            0xFF08...0xFF0E => {} //MMIO
            0xFF0F => self.ic.iflag = val,
            0xFF10...0xFF3F => {} //MMIO
            0xFF40 => self.gpu.write_lcdc_reg(val),
            0xFF41 => self.gpu.write_stat(val),
            0xFF42 => self.gpu.write_scy(val),
            0xFF43 => self.gpu.write_scx(val),
            0xFF44 => self.gpu.write_ly(val),
            0xFF45 => self.gpu.write_lyc(val), 
            0xFF46 => {} //MMIO
            0xFF47 => self.gpu.write_bgp(val), //MMIO
            0xFF48 => self.gpu.write_obp0(val), //MMIO
            0xFF49 => self.gpu.write_obp1(val), //MMIO
            0xFF4A => self.gpu.write_wy(val),
            0xFF4B => self.gpu.write_wx(val),
            0xFF4C...0xFF4F => {} //MMIO
            0xFF50 => self.boot_mode = !(val == 1),
            0xFF51...0xFF7F => {} //MMIO
            0xFF80...0xFFFE => self.zram[addr as usize & 0x7F] = val,
            0xFFFF => self.ic.ie = val,
            _ => panic!("Can't write 0x{:02x} to 0x{:04x}", val, addr),
        }
        if cfg!(debug_assertions) {
            print!("\t0x{:04x}=0x{:02x}", addr, val);
        }
    }

    pub fn readw(&self, addr: u16) -> u16 {
        ((self.readb(addr + 1) as u16) << 8 | (self.readb(addr) as u16))
    }

    pub fn writew(&mut self, addr: u16, val: u16) {
        self.writeb(addr, (val & 0xFF) as u8);
        self.writeb(addr + 1, (val >> 8) as u8);
    }

    pub fn step(&mut self, ticks: u32) -> u32 {
        self.timer.step(ticks, &mut self.ic);
        // TODO, This assumes that gpu and timer stuff takes no ticks...
        ticks
    }
}

impl fmt::Debug for Interconnect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Interconnect")
         .field("boot_mode", &self.boot_mode)
         .field("cart", &self.cart)
         .field("ic", &self.ic)
         .field("timer", &self.timer)
         .field("gpu", &self.gpu)
         .finish()
    }
}
