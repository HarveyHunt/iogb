use std::fmt;
use std::str;
use std::iter;
use std::path;
use std::fs::File;
use std::io::Read;

const ROM_BANK_SZ: usize = 0x4000;
const RAM_BANK_SZ: usize = 0x2000;

// TODO: Add all of the MBCs to here.
#[derive(Debug)]
enum Mbc {
    None,
    One,
}

impl Mbc {
    // TODO: Return an Option, so we can have nice error handling...
    fn from_header(byte: u8) -> Mbc {
        match byte {
            0x00 => Mbc::None,
            0x01 | 0x02 | 0x03  => Mbc::One,
            inv => panic!("Unknown MBC type: 0x{:02x}", inv),
        }
    }
}

pub struct Cartridge {
    pub title: String,
    mbc: Mbc,
    rom: Vec<u8>,
    rom_bank: u8,
    ram: Vec<u8>,
    ram_bank: u8,
    ram_enable: u8,
}

impl Cartridge {
    pub fn new(rom_name: &path::PathBuf) -> Result<Cartridge, String> {
        let buf = match Cartridge::open_rom(rom_name) {
            Ok(b) => b,
            Err(e) => return Err(e),
        };

        let mbc = Mbc::from_header(buf[0x147]);
        let ram_sz = match buf[0x149] {
            0x00 => 0,
            inv => panic!("Unknown ram size: 0x{:02x}", inv),
        };

        let title_buf = buf[0x134..0x143].to_vec();

        let title = match str::from_utf8(&title_buf) {
            Ok(v) => v,
            Err(e) => panic!("Invalid utf8 {}", e),
        };

        Ok(Cartridge {
            title: title.trim_right_matches('\0').to_string(),
            mbc: mbc,
            rom: buf,
            rom_bank: 1,
            ram: iter::repeat(0).take(ram_sz).collect(),
            ram_bank: 0,
            ram_enable: 0,
        })
    }

    fn open_rom(path: &path::PathBuf) -> Result<Vec<u8>, String> {
        let mut data = vec![];
        let mut file = try!(File::open(path).map_err(|e| format!("{}", e)));

        try!(file.read_to_end(&mut data).map_err(|e| format!("{}", e)));
        Ok(data)
    }

    pub fn read_rom(&self, addr: u16) -> u8 {
        let a = match self.mbc {
            Mbc::None => addr as usize,
            Mbc::One => {
                if (addr as usize) < ROM_BANK_SZ {
                    addr as usize
                } else {
                    addr as usize + (self.rom_bank as usize * ROM_BANK_SZ)
                }
            }
        };
        self.rom[a & (self.rom.len() - 1)]
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        if self.ram_enable == 0xA && self.ram.is_empty() {
            self.ram[((addr as usize & (RAM_BANK_SZ - 1)) + (self.ram_bank as usize * RAM_BANK_SZ))]
        } else {
            0 //TODO: Is this correct?
        }
    }

    pub fn write_rom(&self, addr: u16, val: u8) {}
    pub fn write_ram(&self, addr: u16, val: u8) {}
}

impl fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Cartridge")
            .field("title", &self.title)
            .field("mbc", &self.mbc)
            .field("ram_enable", &self.ram_enable)
            .field("ram_bank", &self.ram_bank)
            .field("rom_bank", &self.rom_bank)
            .finish()
    }
}
