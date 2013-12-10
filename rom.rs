use std::io::fs::File;
use std::num;
use addressable::Addressable;

pub struct Rom<ADDR> {
	priv data: ~[u8],
}

impl<ADDR: Int> Rom<ADDR> {
	pub fn new (path: &Path) -> Rom<ADDR> {
		Rom { data: File::open(path).read_to_end() }
	}

	pub fn new_sized (size: uint, path: &Path) -> Rom<ADDR> {
		let rom = Rom::new(path);
		if rom.data.len() != size { fail!("rom: ROM file size does not match expected size (${:x} != ${:x})", rom.data.len(), size); }
		rom
	}

	pub fn size (&self) -> uint {
		self.data.len()
	}
}

impl<ADDR: Int> Addressable<ADDR> for Rom<ADDR> {
	fn get (&self, addr: ADDR) -> u8 {
		let i: uint = num::cast(addr).unwrap();
		if i >= self.data.len() { fail!("rom: Read beyond memory bounds (${:x} >= ${:x})", i, self.data.len()); }
		self.data[i]
	}

	fn set (&mut self, addr: ADDR, _data: u8) {
		let i: uint = num::cast(addr).unwrap();
		warn!("rom: Ignoring write to read-only memory (${:x})", i);
	}
}


#[cfg(test)]
mod test {
	use super::Rom;

	#[test]
	fn test_new () {
		let memory: Rom<u16> = Rom::new(&Path::new("kernal.rom"));
		assert_eq!(memory.size(), 8192);
	}

	#[test]
	fn test_new_sized () {
		let memory: Rom<u16> = Rom::new_sized(8192, &Path::new("kernal.rom"));
		assert_eq!(memory.size(), 8192);
	}

	#[test]
	fn test_read () {
		let memory: Rom<u16> = Rom::new(&Path::new("kernal.rom"));
		assert_eq!(memory.get(0x0123), 0x60);
	}
}
