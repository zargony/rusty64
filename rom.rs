use std::io;
use std::num;
use addressable::Addressable;

pub struct Rom<ADDR> {
	priv data: ~[u8],
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

impl<ADDR: Int> Rom<ADDR> {
	pub fn new (size: uint, path: &Path) -> ~Rom<ADDR> {
		let mut rom = ~Rom { data: ~[] };
		match io::read_whole_file(path) {
			Err(err) => fail!("rom: Error reading ROM file: %?", err),
			Ok(data) => rom.data = data,
		}
		if rom.data.len() != size { fail!("rom: ROM file size does not match expected size ($%x != $%x)", rom.data.len(), size); }
		rom
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test () {
		let memory = Rom::new(8192, &Path("kernal.rom"));
		assert_eq!(memory.get(0x0123), 0x60);
	}
}
