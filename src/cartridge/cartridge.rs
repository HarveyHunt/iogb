use std::iter;
use std::path;
use std::fs::File;
use std::io::Read;

const ROM_BANK_SZ: usize = 0x4000;
const RAM_BANK_SZ: usize = 0x2000;

// TODO: Add all of the MBCs to here.
enum Mbc {
    None,
}

pub struct Cartridge {
    mbc: Mbc,
    rom: Vec<u8>,
    rom_bank: u8,
    ram: Vec<u8>,
    ram_bank: u8,
    ram_enable: u8,
}

impl Cartridge {
    pub fn new(rom_name: &str) -> Cartridge {
        // TODO: Pass this error further up.
        let buf = Cartridge::open_rom(path::PathBuf::from(rom_name)).unwrap();
        // TODO: Maybe pull this buffer parsing into another function?
        let mbc = match buf[0x147] {
            0x00 => Mbc::None,
            inv => panic!("Unknown MBC type: 0x{:x}", inv),
        };

        let ram_sz = match buf[0x149] {
            0x00 => 0,
            inv => panic!("Unknown ram size: 0x{:x}", inv),
        };

        Cartridge {
            mbc: mbc,
            rom: buf,
            rom_bank: 1,
            ram: iter::repeat(0).take(ram_sz).collect(),
            ram_bank: 0,
            ram_enable: 0,
        }
    }

    fn open_rom(path: path::PathBuf) -> Result<Vec<u8>, String> {
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
}
