use std::{num, os};
use std::io::fs::File;
use super::{Addr, Addressable};

/// Generic read-only memory (ROM)
pub struct Rom<A> {
	priv data: ~[u8],
	priv last_addr: A,
}

impl<A: Addr> Rom<A> {
	/// Create new ROM with contents of the given file
	pub fn new (path: &Path) -> Rom<A> {
		let filename = os::self_exe_path().unwrap().join("../share").join(path);
		info!("rom: Loading ROM from {}", filename.display());
		let data = File::open(&filename).read_to_end().unwrap();
		let last_addr: A = num::cast(data.len() - 1).unwrap();
		Rom { data: data, last_addr: last_addr }
	}

	/// Returns the size of the ROM
	#[allow(dead_code)]
	pub fn size (&self) -> uint {
		self.data.len()
	}
}

impl<A: Addr> Addressable<A> for Rom<A> {
	fn get (&self, addr: A) -> u8 {
		if addr > self.last_addr { fail!("rom: Read beyond memory bounds (${:X} > ${:X})", addr, self.last_addr); }
		let i: u64 = num::cast(addr).unwrap();
		self.data[i]
	}

	fn set (&mut self, addr: A, _data: u8) {
		warn!("rom: Ignoring write to read-only memory (${:X})", addr);
	}
}


#[cfg(test)]
mod test {
	use super::super::Addressable;
	use super::Rom;

	#[test]
	fn create_with_file_contents () {
		let memory: Rom<u16> = Rom::new(&Path::new("c64/kernal.rom"));
		assert_eq!(memory.size(), 8192);
	}

	#[test]
	fn read () {
		let memory: Rom<u16> = Rom::new(&Path::new("c64/kernal.rom"));
		assert_eq!(memory.get(0x0123), 0x60);
	}

	#[test]
	fn write_does_nothing () {
		let mut memory: Rom<u16> = Rom::new(&Path::new("c64/kernal.rom"));
		memory.set(0x123, 0x55);
		assert!(memory.get(0x0123) != 0x55);
	}
}
