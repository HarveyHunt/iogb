use std::fmt;
use interrupt;

const VRAM_SZ: usize = 0x2000;
const OAM_SZ: usize = 0xA0;

#[derive(PartialEq, Debug)]
enum Mode {
    HBlank = 0b00,
    VBlank = 0b01,
    AccessingOam = 0b10,
    AccessingVram = 0b11,
}

impl Mode {
    fn as_flag(&self) -> u8 {
        use self::Mode::*;
        match *self {
            HBlank => 0b00,
            VBlank => 0b01,
            AccessingOam => 0b10,
            AccessingVram => 0b11,
        }
    }
}

pub struct Gpu {
    mode: Mode,
    vram: [u8; VRAM_SZ],
    oam: [u8; OAM_SZ],
    lcd_enable: bool,
    obj_on: bool,
    bg_enable: bool,
    stat: u8,
    scroll_x: u8,
    scroll_y: u8,
    win_x: u8,
    win_y: u8,
    ly: u8,
    lyc: u8,
}

// TODO: Display the regs as hex
impl fmt::Debug for Gpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GPU")
         .field("mode", &self.mode)
         .field("lcd_enable", &self.lcd_enable)
         .field("obj_on", &self.obj_on)
         .field("bg_enable", &self.bg_enable)
         .field("lcdc", &format_args!("0x{:02x}", self.read_lcdc_reg()))
         .field("stat", &format_args!("0x{:02x}", self.read_stat()))
         .field("scroll_x", &format_args!("0x{:02x}", self.scroll_x))
         .field("scroll_y", &format_args!("0x{:02x}", self.scroll_y))
         .field("win_x", &format_args!("0x{:02x}", self.win_x))
         .field("win_y", &format_args!("0x{:02x}", self.win_y))
         .field("ly", &format_args!("0x{:02x}", self.ly))
         .field("lyc", &format_args!("0x{:02x}", self.lyc))
         .finish()
    }
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            mode: Mode::AccessingOam,
            vram: [0; VRAM_SZ],
            oam: [0; OAM_SZ],
            lcd_enable: false,
            obj_on: false,
            bg_enable: false,
            stat: 0,
            scroll_x: 0,
            scroll_y: 0,
            win_x: 0,
            win_y: 0,
            ly: 0,
            lyc: 0,
        }
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        // TODO: Handle banking of VRAM
        if self.mode == self::Mode::AccessingVram {
            return 0xFF;
        }
        self.vram[(addr as usize) & VRAM_SZ - 1]
    }

    pub fn write_vram(&mut self, addr: u16, val: u8) {
        // TODO: Handle banking of VRAM
        if self.mode == self::Mode::AccessingVram {
            return;
        }
        self.vram[(addr as usize) & VRAM_SZ - 1] = val;
    }

    pub fn write_lcdc_reg(&mut self, val: u8) {
        let new_lcd_enable = (val & 0x80) != 0;
        if self.lcd_enable && !new_lcd_enable {
            self.ly = 0;
        }
        self.lcd_enable = new_lcd_enable;
        self.obj_on = (val & 0x02) != 0;
        self.bg_enable = (val & 0x01) != 0;
    }

    pub fn read_lcdc_reg(&self) -> u8 {
        return (self.lcd_enable as u8) << 7 | (self.obj_on as u8) << 1 | (self.bg_enable as u8);
    }

    pub fn write_wx(&mut self, val: u8) {
        self.win_x = val;
    }

    pub fn write_wy(&mut self, val: u8) {
        self.win_y = val;
    }

    pub fn read_wx(&self) -> u8 {
        self.win_x
    }

    pub fn read_wy(&self) -> u8 {
        self.win_y
    }

    pub fn write_scx(&mut self, val: u8) {
        self.scroll_x = val;
    }

    pub fn write_scy(&mut self, val: u8) {
        self.scroll_y = val;
    }

    pub fn read_scx(&self) -> u8 {
        self.scroll_x
    }

    pub fn read_scy(&self) -> u8 {
        self.scroll_y
    }

    pub fn read_ly(&self) -> u8 {
        self.ly
    }

    pub fn write_ly(&mut self, val: u8) {
        self.ly = val;
    }

    pub fn read_lyc(&self) -> u8 {
        self.lyc
    }

    pub fn write_lyc(&mut self, val: u8) {
        self.lyc = val;
    }

    pub fn read_stat(&self) -> u8 {
        (self.stat & 0xF8) | ((self.lyc == self.ly) as u8) << 3 | self.mode.as_flag()
    }

    pub fn write_stat(&mut self, val: u8) {
        self.stat = self.stat & 0x0F | val;
    }

    pub fn step(&mut self, cycles: u32, ic: &mut interrupt::InterruptController) {
        if !self.lcd_enable {
            return;
        }

        if self.ly >= 144 {
            self.mode = self::Mode::VBlank;
            ic.request_interrupt(interrupt::Interrupt::VBlank);
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        if self.mode == self::Mode::AccessingVram || self.mode == self::Mode::AccessingOam {
            return 0xFF;
        }
        self.oam[(addr as usize) & OAM_SZ - 1]
    }

    pub fn write_oam(&mut self, addr: u16, val: u8) {
        if self.mode == self::Mode::AccessingVram || self.mode == self::Mode::AccessingOam {
            return;
        }
        self.oam[(addr as usize) & OAM_SZ - 1] = val;
    }
}
