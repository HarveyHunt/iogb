use std::fmt;
use gameboy::{SCREEN_W, SCREEN_H};
use interrupt;

const VRAM_TILES: usize = 384;
const TILE_MAP_SZ: usize = 0x400;
const SPRITE_COUNT: usize = 40;
const HBLANK_CYCLES: i16 = 204;
const ACCESSING_OAM_CYCLES: i16 = 80;
const ACCESSING_VRAM_CYCLES: i16 = 172;
const VBLANK_FULL_LINE_CYCLES: i16 = 576;

#[derive(PartialEq, Debug)]
enum Mode {
    HBlank = 0b00,
    VBlank = 0b01,
    AccessingOam = 0b10,
    AccessingVram = 0b11,
}

bitflags!(
    flags SpriteFlags: u8 {
        const SPRITE_PRIORITY = 1 << 7,
        const SPRITE_Y_FLIP = 1 << 6,
        const SPRITE_X_FLIP = 1 << 5,
        const SPRITE_PALETTE = 1 << 4,
    }
);

bitflags!(
    flags StatReg: u8 {
        const STAT_CMP = 1 << 2,
        const STAT_HBLANK_INT = 1 << 3,
        const STAT_VBLANK_INT = 1 << 4,
        const STAT_OAM_INT = 1 << 5,
        const STAT_CMP_INT = 1 << 6,
    }
);

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

#[derive(Copy, Clone, Default)]
struct Tile {
    pixels: [u8; 16],
}

#[derive(Copy, Clone)]
struct Sprite {
    x: u8,
    y: u8,
    tile_index: u8,
    flags: SpriteFlags,
}

impl Sprite {
    pub fn new() -> Sprite {
        Sprite {
            x: 0,
            y: 0,
            tile_index: 0,
            flags: SpriteFlags::empty(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Colour {
    White = 0,
    LightGrey = 1,
    DarkGrey = 2,
    Black = 3,
}

impl Colour {
    fn from_bits(col: u8) -> Colour {
        use self::Colour::*;
        match col {
            0 => White,
            1 => LightGrey,
            2 => DarkGrey,
            3 => Black,
            _ => panic!("Invalid colour shade 0x{:02x}", col),
        }
    }
}

#[derive(Debug)]
struct Palette {
    darkest: Colour,
    dark: Colour,
    light: Colour,
    lightest: Colour,
    reg: u8,
}

impl Palette {
    fn new() -> Palette {
        use self::Colour::*;
        Palette {
            reg: 0,
            darkest: White,
            dark: White,
            light: White,
            lightest: White,
        }
    }

    fn lookup(&self, colour: &Colour) -> Colour {
        use self::Colour::*;
        match *colour {
            Black => self.darkest,
            DarkGrey => self.dark,
            LightGrey => self.light,
            White => self.lightest,
        }
    }

    fn set_reg(&mut self, val: u8) {
        self.reg = val;
        self.darkest = Colour::from_bits((val >> 6) & 0x03);
        self.dark = Colour::from_bits((val >> 4) & 0x03);
        self.light = Colour::from_bits((val >> 2) & 0x03);
        self.lightest = Colour::from_bits(val & 0x03);
    }
}

pub struct Gpu {
    mode: Mode,
    ticks: i16,
    oam: [Sprite; SPRITE_COUNT],
    // For a CGB, this should be [W * H * 3] to account for RGB.
    pub buffer: [u8; SCREEN_W * SCREEN_H],
    lcd_enable: bool,
    win_tile_map: bool,
    win_enable: bool,
    bg_tile_set: bool,
    bg_tile_map: bool,
    obj_size: u8, // 8x8 or 8x16
    obj_enable: bool,
    bg_enable: bool,
    stat: StatReg,
    scroll_x: u8,
    scroll_y: u8,
    win_x: u8,
    win_y: u8,
    ly: u8,
    lyc: u8,
    bgp: self::Palette,
    obp0: self::Palette,
    obp1: self::Palette,
    tile_set: [Tile; VRAM_TILES],
    tile_map1: [u8; TILE_MAP_SZ],
    tile_map2: [u8; TILE_MAP_SZ],
}

// TODO: Display the regs as hex
impl fmt::Debug for Gpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GPU")
            .field("mode", &self.mode)
            .field("lcd_enable", &self.lcd_enable)
            .field("win_tile_map", &self.win_tile_map)
            .field("win_enable", &self.win_enable)
            .field("bg_tile_set", &self.bg_tile_set)
            .field("bg_tile_map", &self.bg_tile_map)
            .field("obj_size", &self.obj_size)
            .field("obj_enable", &self.obj_enable)
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
            ticks: ACCESSING_OAM_CYCLES,
            mode: Mode::AccessingOam,
            oam: [Sprite::new(); SPRITE_COUNT],
            buffer: [0; SCREEN_W * SCREEN_H],
            lcd_enable: false,
            win_tile_map: false,
            win_enable: false,
            bg_tile_set: false,
            bg_tile_map: false,
            obj_size: 8,
            obj_enable: false,
            bg_enable: false,
            stat: StatReg::empty(),
            scroll_x: 0,
            scroll_y: 0,
            win_x: 0,
            win_y: 0,
            ly: 0,
            lyc: 0,
            bgp: Palette::new(),
            obp0: Palette::new(),
            obp1: Palette::new(),
            tile_set: [Tile::default(); VRAM_TILES],
            tile_map1: [0; TILE_MAP_SZ],
            tile_map2: [0; TILE_MAP_SZ],
        }
    }

    pub fn read_tileset(&self, addr: u16) -> u8 {
        if self.mode == self::Mode::AccessingVram {
            return 0xFF;
        }
        let tile = &self.tile_set[addr as usize >> 4];
        tile.pixels[addr as usize % 16]
    }

    pub fn write_tileset(&mut self, addr: u16, val: u8) {
        if self.mode == self::Mode::AccessingVram {
            return;
        }
        let tile = &mut self.tile_set[addr as usize >> 4];
        tile.pixels[addr as usize % 16] = val;
    }

    pub fn read_tilemap1(&self, addr: u16) -> u8 {
        if self.mode == self::Mode::AccessingVram {
            return 0xFF;
        }
        self.tile_map1[addr as usize]
    }

    pub fn write_tilemap1(&mut self, addr: u16, val: u8) {
        if self.mode == self::Mode::AccessingVram {
            return;
        }
        self.tile_map1[addr as usize] = val;
    }

    pub fn read_tilemap2(&self, addr: u16) -> u8 {
        if self.mode == self::Mode::AccessingVram {
            return 0xFF;
        }
        self.tile_map2[addr as usize]
    }

    pub fn write_tilemap2(&mut self, addr: u16, val: u8) {
        if self.mode == self::Mode::AccessingVram {
            return;
        }
        self.tile_map2[addr as usize] = val;
    }

    pub fn write_lcdc_reg(&mut self, val: u8) {
        let new_lcd_enable = (val & 0x80) != 0;
        if self.lcd_enable && !new_lcd_enable {
            self.ly = 0;
        }
        self.lcd_enable = new_lcd_enable;
        self.win_tile_map = (val & 0x40) != 0;
        self.win_enable = (val & 0x20) != 0;
        self.bg_tile_set = (val & 0x10) != 0;
        self.bg_tile_map = (val & 0x08) != 0;
        self.obj_size = ((val & 0x04) == 16) as u8;
        self.obj_enable = (val & 0x02) != 0;
        self.bg_enable = (val & 0x01) != 0;
    }

    pub fn read_lcdc_reg(&self) -> u8 {
        return (self.lcd_enable as u8) << 7 | (self.win_tile_map as u8) << 6 |
               (self.win_enable as u8) << 5 | (self.bg_tile_set as u8) << 4 |
               (self.bg_tile_map as u8) << 3 | ((self.obj_size == 16) as u8) << 2 |
               (self.obj_enable as u8) << 1 |
               (self.bg_enable as u8);
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
        self.stat.bits | self.mode.as_flag()
    }

    pub fn write_stat(&mut self, val: u8) {
        let nstat = StatReg::from_bits_truncate(val);
        self.stat = (self.stat & STAT_CMP) | (nstat);
    }

    pub fn read_bgp(&self) -> u8 {
        self.bgp.reg
    }

    pub fn read_obp0(&self) -> u8 {
        self.obp0.reg
    }

    pub fn read_obp1(&self) -> u8 {
        self.obp1.reg
    }

    pub fn write_bgp(&mut self, val: u8) {
        self.bgp.set_reg(val);
    }

    pub fn write_obp0(&mut self, val: u8) {
        self.obp0.set_reg(val);
    }

    pub fn write_obp1(&mut self, val: u8) {
        self.obp1.set_reg(val);
    }

    fn check_cmp_int(&mut self, ic: &mut interrupt::InterruptController) {
        if self.ly != self.lyc {
            self.stat.remove(STAT_CMP);
        } else {
            self.stat.insert(STAT_CMP);
            if self.stat.contains(STAT_CMP_INT) {
                ic.request_interrupt(interrupt::Interrupt::LCDCStat);
            }
        }
    }

    fn change_mode(&mut self, mode: self::Mode, ic: &mut interrupt::InterruptController) {
        self.mode = mode;
        match self.mode {
            Mode::HBlank => self.ticks += HBLANK_CYCLES,
            Mode::VBlank => {
                self.ticks += VBLANK_FULL_LINE_CYCLES;
                ic.request_interrupt(interrupt::Interrupt::VBlank);
                if self.stat.contains(STAT_VBLANK_INT) {
                    ic.request_interrupt(interrupt::Interrupt::LCDCStat);
                }
            }
            Mode::AccessingOam => {
                self.ticks += ACCESSING_OAM_CYCLES;
                if self.stat.contains(STAT_OAM_INT) {
                    ic.request_interrupt(interrupt::Interrupt::LCDCStat);
                }
            }
            Mode::AccessingVram => self.ticks += ACCESSING_VRAM_CYCLES,
        }
    }

    pub fn step(&mut self, cycles: u32, ic: &mut interrupt::InterruptController) {
        if !self.lcd_enable {
            return;
        }

        self.ticks -= cycles as i16;

        // We haven't finished our current mode!
        if self.ticks > 0 {
            return;
        }

        match self.mode {
            Mode::HBlank => {
                self.ly += 1;
                if self.ly >= SCREEN_H as u8 {
                    self.change_mode(self::Mode::VBlank, ic);
                } else {
                    self.change_mode(self::Mode::AccessingOam, ic);
                }
                self.check_cmp_int(ic);
            }
            Mode::VBlank => {
                self.ly += 1;
                if self.ly <= 153 {
                    self.ticks += VBLANK_FULL_LINE_CYCLES;
                } else {
                    self.ly = 0;
                    self.change_mode(self::Mode::AccessingOam, ic);
                }
                self.check_cmp_int(ic);
            }
            Mode::AccessingOam => {
                self.change_mode(self::Mode::AccessingVram, ic);
            }
            Mode::AccessingVram => {
                self.render_line();
                self.change_mode(self::Mode::HBlank, ic);
            }
        }
    }

    pub fn render_line(&mut self) {
        let start = self.ly as usize * SCREEN_W;
        let end = start + SCREEN_W;
        self.render_background(start, end);
        self.render_window(start, end);
        self.render_sprites(start, end);
    }

    fn render_background(&mut self, start: usize, end: usize) {
        if !self.bg_enable {
            return;
        }
        let y = self.ly.wrapping_add(self.scroll_y);
        let row = (y >> 3) as usize;
        let tile_map = if !self.bg_tile_map {
            &self.tile_map1
        } else {
            &self.tile_map2
        };

        for i in 0..SCREEN_W {
            let x = i.wrapping_add(self.scroll_x as usize);
            let bit = x % 8;
            let column = x >> 3;
            // The first of two rows in the tile that we are going to render.
            let tile_row = (y % 8) * 2;

            // Find the required tile...
            let tile_num = tile_map[(row * 32) + column] as usize;
            let tile = if self.bg_tile_set {
                &self.tile_set[tile_num]
            } else {
                // FIXME: I'm not convinced this is correct...
                &self.tile_set[(tile_num as i8 as i16 + 128) as usize]
            };

            // The 2 bytes that we are going to use to calculate a colour value.
            // 1 - x - x - - - - 0 1 2 3 0 0 0 0
            // 2 - - x x - - - - 0 1 2 3 0 0 0 0
            let row_data = (tile.pixels[tile_row as usize], tile.pixels[tile_row as usize + 1]);

            // Bit denotes the column in the row_data we are going to use
            let colour_number = (((row_data.0 >> bit) & 1) * 2) | ((row_data.1 >> bit) & 1);
            // Convert the raw colour number into a Colour and look it up in the
            // BGP so that we can get the _real_ Colour.
            let palette_colour = Colour::from_bits(colour_number);
            let colour = self.bgp.lookup(&palette_colour);

            // We are going to convert this "colour" into a true RGB colour
            // using a LUT that is passed into the fragment shader. We could
            // iterate through the buffer and create a new buffer with RGB,
            // but it'd be quicker and more memory efficient to just do it in
            // the shader.
            self.buffer[start + i] = colour as u8;
        }
    }

    fn render_window(&mut self, start: usize, end: usize) {
        if !self.win_enable {
            return;
        }
    }

    fn render_sprites(&mut self, start: usize, end: usize) {
        if !self.obj_enable {
            return;
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        if self.mode == self::Mode::AccessingVram || self.mode == self::Mode::AccessingOam {
            return 0xFF;
        }
        let sprite = &self.oam[addr as usize >> 2];
        match addr % 4 {
            0 => sprite.y,
            1 => sprite.x,
            2 => sprite.tile_index,
            3 => sprite.flags.bits(),
            _ => panic!(),
        }
    }

    pub fn write_oam(&mut self, addr: u16, val: u8) {
        if self.mode == self::Mode::AccessingVram || self.mode == self::Mode::AccessingOam {
            return;
        }
        let sprite = &mut self.oam[addr as usize >> 2];
        match addr % 4 {
            0 => sprite.y = val,
            1 => sprite.x = val,
            2 => sprite.tile_index = val,
            3 => sprite.flags = SpriteFlags::from_bits_truncate(val),
            _ => panic!(),
        }
    }
}
