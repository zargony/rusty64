use std::num;
use std::vec;
use addressable::Addressable;

pub struct Ram<ADDR> {
	data: ~[u8],
}

impl<ADDR: Int> Ram<ADDR> {
	pub fn new (size: uint) -> ~Ram<ADDR> {
		~Ram { data: vec::from_elem(size, 0) }
	}
}

impl<ADDR: Int> Addressable<ADDR, u8> for Ram<ADDR> {
	pub fn get (&self, addr: ADDR) -> u8 {
		let i: uint = num::cast(addr);
		if i >= self.data.len() { fail!("memory: read beyond memory bounds ($%x >= $%x)", i, self.data.len()); }
		self.data[i]
	}

	pub fn set (&mut self, addr: ADDR, data: u8) {
		let i: uint = num::cast(addr);
		if i >= self.data.len() { fail!("memory: write beyond memory bounds ($%x >= $%x)", i, self.data.len()); }
		self.data[i] = data;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test () {
		let mut memory = Ram::new(1024);
		memory.set(0x0123, 0x55);
		assert_eq!(memory.get(0x0123), 0x55);
	}
}
