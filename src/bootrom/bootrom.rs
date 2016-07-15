use std::fs::File;
use std::path;
use std::io::Read;

// TODO: Move these constants into their own file
const BOOTROM_SZ: usize = 0x100;

pub fn load_bootrom(path: &path::PathBuf) -> Result<Vec<u8>, String> {
    let mut buf = [0; BOOTROM_SZ];

    let mut file = try!(File::open(path).map_err(|e| format!("{}", e)));

    try!(file.read_exact(&mut buf).map_err(|e| format!("{}", e)));

    Ok(buf.to_vec())
}
