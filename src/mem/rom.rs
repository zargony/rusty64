use std::env;
use std::io::Read;
use std::fs::File;
use std::path::Path;
use num;
use super::{Address, Addressable};

/// Generic read-only memory (ROM)
pub struct Rom<A> {
    data: Vec<u8>,
    last_addr: A,
}

impl<A: Address> Rom<A> {
    /// Create new ROM with contents of the given file
    pub fn new<P: AsRef<Path>> (path: P) -> Rom<A> {
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
        let last_addr: A = num::cast(len - 1).unwrap();
        Rom { data: data, last_addr: last_addr }
    }

    /// Returns the capacity of the ROM
    #[allow(dead_code)]
    pub fn capacity (&self) -> usize {
        self.data.len()
    }
}

impl<A: Address> Addressable<A> for Rom<A> {
    fn get (&self, addr: A) -> u8 {
        if addr > self.last_addr {
            panic!("rom: Read beyond memory bounds ({} > {})", addr.display(), self.last_addr.display());
        }
        let i: usize = num::cast(addr).unwrap();
        self.data[i]
    }

    fn set (&mut self, addr: A, _data: u8) {
        warn!("rom: Ignoring write to read-only memory ({})", addr.display());
    }
}


#[cfg(test)]
mod tests {
    use super::super::Addressable;
    use super::*;

    #[test]
    fn create_with_file_contents () {
        let memory: Rom<u16> = Rom::new("c64/kernal.rom");
        assert_eq!(memory.capacity(), 8192);
    }

    #[test]
    fn read () {
        let memory: Rom<u16> = Rom::new("c64/kernal.rom");
        assert_eq!(memory.get(0x0123), 0x60);
    }

    #[test]
    fn write_does_nothing () {
        let mut memory: Rom<u16> = Rom::new("c64/kernal.rom");
        memory.set(0x0123, 0x55);
        assert!(memory.get(0x0123) != 0x55);
    }
}
