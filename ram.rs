use std::num;
use std::vec;
use addressable::Addressable;

pub struct Ram<ADDR> {
	priv data: ~[u8],
}

impl<ADDR: Int> Ram<ADDR> {
	pub fn new () -> Ram<ADDR> {
		let last_addr: uint = num::cast(num::Bounded::max_value::<ADDR>());
		Ram::new_sized(last_addr + 1)
	}

	pub fn new_sized (size: uint) -> Ram<ADDR> {
		Ram { data: vec::from_elem(size, 0) }
	}

	pub fn size (&self) -> uint {
		self.data.len()
	}
}

impl<ADDR: Int> Addressable<ADDR> for Ram<ADDR> {
	pub fn get (&self, addr: ADDR) -> u8 {
		let i: uint = num::cast(addr);
		if i >= self.data.len() { fail!("ram: Read beyond memory bounds ($%x >= $%x)", i, self.data.len()); }
		self.data[i]
	}

	pub fn set (&mut self, addr: ADDR, data: u8) {
		let i: uint = num::cast(addr);
		if i >= self.data.len() { fail!("ram: Write beyond memory bounds ($%x >= $%x)", i, self.data.len()); }
		self.data[i] = data;
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new () {
		let memory: Ram<u8> = Ram::new();
		assert_eq!(memory.size(), 256);
	}

	#[test]
	fn test_new_sized () {
		let memory: Ram<u16> = Ram::new_sized(1024);
		assert_eq!(memory.size(), 1024);
	}

	#[test]
	fn test_read_write () {
		let mut memory: Ram<u16> = Ram::new_sized(1024);
		memory.set(0x0123, 0x55);
		assert_eq!(memory.get(0x0123), 0x55);
	}
}
