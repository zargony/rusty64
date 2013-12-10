use std::num;
use std::vec;
use addressable::Addressable;

pub struct Ram<ADDR> {
	priv data: ~[u8],
}

impl<ADDR: Int> Ram<ADDR> {
	pub fn new () -> Ram<ADDR> {
		let last_addr: ADDR = num::Bounded::max_value();
		Ram::new_sized(1u + num::cast(last_addr).unwrap())
	}

	pub fn new_sized (size: uint) -> Ram<ADDR> {
		Ram { data: vec::from_elem(size, 0u8) }
	}

	pub fn size (&self) -> uint {
		self.data.len()
	}
}

impl<ADDR: Int> Addressable<ADDR> for Ram<ADDR> {
	fn get (&self, addr: ADDR) -> u8 {
		let i: uint = num::cast(addr).unwrap();
		if i >= self.data.len() { fail!("ram: Read beyond memory bounds (${:x} >= ${:x})", i, self.data.len()); }
		self.data[i]
	}

	fn set (&mut self, addr: ADDR, data: u8) {
		let i: uint = num::cast(addr).unwrap();
		if i >= self.data.len() { fail!("ram: Write beyond memory bounds (${:x} >= ${:x})", i, self.data.len()); }
		self.data[i] = data;
	}
}


#[cfg(test)]
mod test {
	use super::Ram;

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
