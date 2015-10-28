//!
//! Read Only Memory (ROM)
//!

use std::env;
use std::io::Read;
use std::fs::File;
use std::path::Path;
use addr::Address;
use mem::Addressable;

/// Generic read-only memory (ROM)
pub struct Rom {
    data: Vec<u8>,
    last_addr: u16,
}

impl Rom {
    /// Create new ROM with contents of the given file
    pub fn new<P: AsRef<Path>> (path: P) -> Rom {
        let filename = env::current_dir().unwrap().join("share").join(path);
        info!("rom: Loading ROM from {}", filename.display());
        let mut data = Vec::new();
        let mut f = match File::open(&filename) {
            Err(err) => panic!("rom: Unable to open ROM: {}", err),
            Ok(f) => f,
        };
        let len = match f.read_to_end(&mut data) {
            Err(err) => panic!("rom: Unable to load ROM: {}", err),
            Ok(0) => panic!("rom: Unable to load empty ROM"),
            Ok(len) => len,
        };
        Rom { data: data, last_addr: (len - 1) as u16 }
    }

    /// Returns the capacity of the ROM
    pub fn capacity (&self) -> usize {
        self.data.len()
    }
}

impl Addressable for Rom {
    fn get<A: Address> (&self, addr: A) -> u8 {
        if addr.to_u16() > self.last_addr {
            panic!("rom: Read beyond memory bounds ({} > {})", addr.display(), self.last_addr.display());
        }
        unsafe { self.data[addr.to_usize()] }
    }

    fn set<A: Address> (&mut self, addr: A, _data: u8) {
        warn!("rom: Ignoring write to read-only memory ({})", addr.display());
    }
}


#[cfg(test)]
mod tests {
    use mem::Addressable;
    use super::*;

    #[test]
    fn create_with_file_contents () {
        let memory = Rom::new("c64/kernal.rom");
        assert_eq!(memory.capacity(), 8192);
    }

    #[test]
    fn read () {
        let memory = Rom::new("c64/kernal.rom");
        assert_eq!(memory.get(0x0123), 0x60);
    }

    #[test]
    fn write_does_nothing () {
        let mut memory = Rom::new("c64/kernal.rom");
        memory.set(0x0123, 0x55);
        assert!(memory.get(0x0123) != 0x55);
    }
}
