use std::fmt;

use interrupt;
use cartridge;
use timer;
use gpu;
use bootrom;

const WRAM_SZ: usize = 0x2000;
const ZRAM_SZ: usize = 0x7F;

pub struct Interconnect {
    pub brom: bootrom::Bootrom, // 0x0000 -> 0x00FF
    wram: [u8; WRAM_SZ], // 0xC000 -> 0xDFFF, shadowed @ 0xE000 -> 0xFDFF
    zram: [u8; ZRAM_SZ], // 0xFF80 -> 0xFFFF
    cart: cartridge::Cartridge,
    boot_mode: bool, // Map brom into bottom of memory?
    // TODO: Make this private and implement wrapper functions
    pub ic: interrupt::InterruptController,
    pub timer: timer::Timer,
    pub gpu: gpu::Gpu,
}

impl Interconnect {
    pub fn new(cart: cartridge::Cartridge, bootrom: bootrom::Bootrom) -> Interconnect {
        let mut ic = Interconnect {
            brom: bootrom,
            wram: [0; WRAM_SZ],
            zram: [0; ZRAM_SZ],
            cart: cart,
            boot_mode: true,
            ic: interrupt::InterruptController::new(),
            timer: timer::Timer::new(),
            gpu: gpu::Gpu::new(),
        };

        if !ic.brom.is_used() {
            ic.fake_boot_rom();
        }
        ic
    }

    pub fn fake_boot_rom(&mut self) {
        // Taken from the legendary pandocs.
        // http://bgb.bircd.org/pandocs.htm
        self.writew(0xFF05, 0x00);   // TIMA
        self.writew(0xFF06, 0x00);   // TMA
        self.writew(0xFF07, 0x00);   // TAC
        self.writew(0xFF10, 0x80);   // NR10
        self.writew(0xFF11, 0xBF);   // NR11
        self.writew(0xFF12, 0xF3);   // NR12
        self.writew(0xFF14, 0xBF);   // NR14
        self.writew(0xFF16, 0x3F);   // NR21
        self.writew(0xFF17, 0x00);   // NR22
        self.writew(0xFF19, 0xBF);   // NR24
        self.writew(0xFF1A, 0x7F);   // NR30
        self.writew(0xFF1B, 0xFF);   // NR31
        self.writew(0xFF1C, 0x9F);   // NR32
        self.writew(0xFF1E, 0xBF);   // NR33
        self.writew(0xFF20, 0xFF);   // NR41
        self.writew(0xFF21, 0x00);   // NR42
        self.writew(0xFF22, 0x00);   // NR43
        self.writew(0xFF23, 0xBF);   // NR30
        self.writew(0xFF24, 0x77);   // NR50
        self.writew(0xFF25, 0xF3);   // NR51
        self.writew(0xFF26, 0xF1);   // NR52
        self.writew(0xFF40, 0x91);   // LCDC
        self.writew(0xFF42, 0x00);   // SCY
        self.writew(0xFF43, 0x00);   // SCX
        self.writew(0xFF45, 0x00);   // LYC
        self.writew(0xFF47, 0xFC);   // BGP
        self.writew(0xFF48, 0xFF);   // OBP0
        self.writew(0xFF49, 0xFF);   // OBP1
        self.writew(0xFF4A, 0x00);   // WY
        self.writew(0xFF4B, 0x00);   // WX
        self.writew(0xFFFF, 0x00);   // IE
    }

    pub fn readb(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x00FF => {
                if self.boot_mode {
                    self.brom.readb(addr)
                } else {
                    self.cart.read_rom(addr)
                }
            }
            0x0100...0x7FFF => self.cart.read_rom(addr),
            0x8000...0x97FF => self.gpu.read_tileset(addr & 0x17FF),
            0x9800...0x9BFF => self.gpu.read_tilemap1(addr & 0x03FF),
            0x9C00...0x9FFF => self.gpu.read_tilemap2(addr & 0x03FF),
            0xA000...0xBFFF => self.cart.read_ram(addr),
            // TODO: 0xD000 -> 0xDFFF is banked on CGB
            0xC000...0xDFFF => self.wram[addr as usize & 0x1FFF],
            0xE000...0xFDFF => self.wram[addr as usize & 0x1FFF],
            0xFE00...0xFE9F => self.gpu.read_oam(addr & 0x9F),
            0xFEA0...0xFEFF => 0, // Gap
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
            0x8000...0x97FF => self.gpu.write_tileset(addr & 0x17FF, val),
            0x9800...0x9BFF => self.gpu.write_tilemap1(addr & 0x03FF, val),
            0x9C00...0x9FFF => self.gpu.write_tilemap2(addr & 0x03FF, val),
            0xA000...0xBFFF => self.cart.write_ram(addr, val),
            // TODO: 0xD000 -> 0xDFFF is banked on CGB
            0xC000...0xDFFF => self.wram[addr as usize & 0x1FFF] = val,
            0xE000...0xFDFF => self.wram[addr as usize & 0x1FFF] = val,
            0xFE00...0xFE9F => self.gpu.write_oam(addr & 0x9F, val),
            0xFEA0...0xFEFF => {} // Gap
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
            0xFFFF => self.ic.enable_all_interrupts(val),
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
        self.gpu.step(ticks, &mut self.ic);
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
