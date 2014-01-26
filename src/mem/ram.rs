use std::{num, vec};
use super::{Addr, Addressable};

/// Generic read/write memory (RAM)
pub struct Ram<A> {
	priv data: ~[u8],
	priv last_addr: A,
}

impl<A: Addr> Ram<A> {
	/// Create new RAM with full capacity of its address range
	pub fn new () -> Ram<A> {
		Ram::with_capacity(num::Bounded::max_value())
	}

	/// Create new RAM which will be addressable from 0 to the given address
	pub fn with_capacity (last_addr: A) -> Ram<A> {
		let size: uint = 1 + num::cast(last_addr.clone()).unwrap();
		Ram { data: vec::from_elem(size, 0u8), last_addr: last_addr }
	}

	/// Returns the size of the RAM
	pub fn size (&self) -> uint {
		self.data.len()
	}
}

impl<A: Addr> Addressable<A> for Ram<A> {
	fn get (&self, addr: A) -> u8 {
		if addr > self.last_addr { fail!("ram: Read beyond memory bounds (${:X} > ${:X})", addr, self.last_addr); }
		let i: u64 = num::cast(addr).unwrap();
		self.data[i]
	}

	fn set (&mut self, addr: A, data: u8) {
		if addr > self.last_addr { fail!("ram: Write beyond memory bounds (${:X} > ${:X})", addr, self.last_addr); }
		let i: u64 = num::cast(addr).unwrap();
		self.data[i] = data;
	}
}


#[cfg(test)]
mod test {
	use super::super::Addressable;
	use super::Ram;

	#[test]
	fn create_with_full_addressable_capacity () {
		let memory: Ram<u8> = Ram::new();
		assert_eq!(memory.size(), 256);
	}

	#[test]
	fn create_with_requested_capacity () {
		let memory: Ram<u16> = Ram::with_capacity(0x03ff_u16);
		assert_eq!(memory.size(), 1024);
	}

	#[test]
	fn read_write () {
		let mut memory: Ram<u16> = Ram::with_capacity(0x03ff_u16);
		memory.set(0x0123, 0x55);
		assert_eq!(memory.get(0x0123), 0x55);
	}
}
