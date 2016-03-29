use interrupt;

const VRAM_SZ: usize = 0x2000;
const OAM_SZ: usize = 0xA0;

#[derive(PartialEq)]
enum Mode {
    HBlank,
    VBlank,
    AccessingOam,
    AccessingVram,
}

pub struct Gpu {
    mode: Mode,
    vram: [u8; VRAM_SZ],
    oam: [u8; OAM_SZ],
    lcd_enable: bool,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            mode: Mode::AccessingOam,
            vram: [0; VRAM_SZ],
            oam: [0; OAM_SZ],
            lcd_enable: false,
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
        self.lcd_enable = (val & 0x80) != 0;
    }

    pub fn read_lcdc_reg(&self) -> u8 {
        return (self.lcd_enable as u8) << 7;
    }

    pub fn step(&mut self, cycles: u32, ic: &mut interrupt::InterruptController) {
        if !self.lcd_enable {
            return;
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
