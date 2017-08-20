use std::fs::File;
use std::path;
use std::io::Read;

// TODO: Move these constants into their own file
const BOOTROM_SZ: usize = 0x100;

pub struct Bootrom {
    buf: Option<Vec<u8>>,
}

impl Bootrom {
    pub fn from_pathbuf(path: Option<&path::PathBuf>) -> Result<Bootrom, String> {
        match path {
            Some(p) => {
                let mut buf = [0; BOOTROM_SZ];
                let mut file = try!(File::open(p).map_err(|e| format!("{}", e)));
                try!(file.read_exact(&mut buf).map_err(|e| format!("{}", e)));
                Ok(Bootrom { buf: Some(buf.to_vec()) })
            }
            None => Ok(Bootrom { buf: None }),
        }
    }

    pub fn readb(&self, addr: u16) -> u8 {
        match self.buf {
            Some(ref b) => b[addr as usize],
            None => 0xff,
        }
    }

    pub fn is_used(&self) -> bool {
        match self.buf {
            Some(ref _b) => true,
            None => false,
        }
    }
}
