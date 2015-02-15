use std::{env, num};
use std::io::Read;
use std::fs::File;
// FIXME: Remove import of std::path::Path once it landed in the prelude
use std::path::{Path, AsPath};
use super::{Address, Addressable};

/// Generic read-only memory (ROM)
pub struct Rom<A> {
    data: Vec<u8>,
    last_addr: A,
}

impl<A: Address> Rom<A> {
    /// Create new ROM with contents of the given file
    pub fn new<P: AsPath + ?Sized> (path: &P) -> Rom<A> {
        // FIXME: Remove explicit Path::new once std::env uses the new std::path module
        let filename = Path::new(&env::current_dir().unwrap()).join("share").join(path);
        info!("rom: Loading ROM from {}", filename.display());
        let mut data = Vec::new();
        let mut f = match File::open(&filename) {
            Err(err) => panic!("rom: Unable to open ROM: {}", err),
            Ok(f) => f,
        };
        match f.read_to_end(&mut data) {
            Err(err) => panic!("rom: Unable to load ROM: {}", err),
            Ok(()) => assert!(data.len() > 0, "rom: Unable to load empty ROM"),
        }
        let last_addr: A = num::cast(data.len() - 1).unwrap();
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
mod test {
    use super::super::Addressable;
    use super::Rom;

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
