use std::num;
use addressable::Addressable;

pub struct TestMemory<ADDR>;

impl<ADDR: Int> TestMemory<ADDR> {
	pub fn new () -> TestMemory<ADDR> {
		TestMemory
	}
}

impl<ADDR: Int> Addressable<ADDR> for TestMemory<ADDR> {
	fn get (&self, addr: ADDR) -> u8 {
		let i: uint = num::cast(addr);
		(i & 0xff) as u8 + (i >> 8) as u8
	}

	fn set (&mut self, addr: ADDR, data: u8) {
		let i: uint = num::cast(addr);
		let expected = (i & 0xff) as u8 + (i >> 8) as u8;
		if data != expected {
			fail!("testmemory: Illegal data write to $%x ($%x != $%x)", i, data as uint, expected as uint);
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_read () {
		let memory: TestMemory<u16> = TestMemory::new();
		assert_eq!(memory.get(0x0012), 0x12);
		assert_eq!(memory.get(0x0123), 0x24);
		assert_eq!(memory.get(0x1234), 0x46);
	}

	#[test]
	fn test_write () {
		let mut memory: TestMemory<u16> = TestMemory::new();
		memory.set(0x0012, 0x12);
		memory.set(0x0123, 0x24);
		memory.set(0x1234, 0x46);
	}
}
