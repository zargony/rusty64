use std::vec;
use addressable::Addressable;

pub struct RAM {
	data: ~[u8],
}

impl RAM {
	pub fn new (size: uint) -> ~RAM {
		~RAM { data: vec::from_elem(size, 0) }
	}
}

impl Addressable<uint, u8> for RAM {
	pub fn get (&self, addr: uint) -> u8 {
		if addr > self.data.len()-1 { fail!("memory: read beyond memory bounds (%? > %?)", addr, self.data.len()-1); }
		self.data[addr]
	}

	pub fn set (&mut self, addr: uint, data: u8) {
		if addr > self.data.len()-1 { fail!("memory: write beyond memory bounds (%? > %?)", addr, self.data.len()-1); }
		self.data[addr] = data;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test () {
		let mut memory = RAM::new(1024);
		memory.set(0x0123, 0x55);
		assert_eq!(memory.get(0x0123), 0x55);
	}
}
