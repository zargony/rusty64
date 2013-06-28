use std::io;
use std::num;
use addressable::Addressable;

pub struct Rom<ADDR> {
	priv data: ~[u8],
}

impl<ADDR: Int> Rom<ADDR> {
	pub fn new (path: &Path) -> Rom<ADDR> {
		let mut rom = Rom { data: ~[] };
		match io::read_whole_file(path) {
			Err(err) => fail!("rom: Error reading ROM file: %?", err),
			Ok(data) => rom.data = data,
		}
		rom
	}

	pub fn new_sized (size: uint, path: &Path) -> Rom<ADDR> {
		let rom = Rom::new(path);
		if rom.data.len() != size { fail!("rom: ROM file size does not match expected size ($%x != $%x)", rom.data.len(), size); }
		rom
	}

	pub fn size (&self) -> uint {
		self.data.len()
	}
}

impl<ADDR: Int> Addressable<ADDR> for Rom<ADDR> {
	pub fn get (&self, addr: ADDR) -> u8 {
		let i: uint = num::cast(addr);
		if i >= self.data.len() { fail!("rom: Read beyond memory bounds ($%x >= $%x)", i, self.data.len()); }
		self.data[i]
	}

	pub fn set (&mut self, addr: ADDR, _data: u8) {
		let i: uint = num::cast(addr);
		warn!("rom: Ignoring write to read-only memory ($%x)", i);
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new () {
		let memory: Rom<u16> = Rom::new(&Path("kernal.rom"));
		assert_eq!(memory.size(), 8192);
	}

	#[test]
	fn test_new_sized () {
		let memory: Rom<u16> = Rom::new_sized(8192, &Path("kernal.rom"));
		assert_eq!(memory.size(), 8192);
	}

	#[test]
	fn test_read () {
		let memory: Rom<u16> = Rom::new(&Path("kernal.rom"));
		assert_eq!(memory.get(0x0123), 0x60);
	}
}
