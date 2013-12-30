use std::num;
use super::{Addr, Addressable};

/// For every address, TestMemory has fixed calculated data contents. When
/// reading, the memory returns this fixed data. When writing, the memory
/// expects this fixed data to be written.
/// See data_for_addr on how the data content is calculated.
pub struct TestMemory<A>;

/// Calculates the data content for a given address within the test memory.
/// Data is calculated by interpreting bits 0-7 and bits 8-15 as u8 numbers
/// and adding them together. E.g. address 0x1234 holds data 0x46 (the sum
/// of 0x12 and 0x34).
fn data_for_addr<A: Addr> (addr: A) -> u8 {
	let i: u64 = num::cast(addr).unwrap();
	i as u8 + (i >> 8) as u8
}

impl<A: Addr> TestMemory<A> {
	/// Create new test memory
	pub fn new () -> TestMemory<A> {
		TestMemory
	}
}

impl<A: Addr> Addressable<A> for TestMemory<A> {
	fn get (&self, addr: A) -> u8 {
		data_for_addr(addr)
	}

	fn set (&mut self, addr: A, data: u8) {
		let expected = data_for_addr(addr.clone());
		if data != expected {
			fail!("testmemory: Illegal data write to ${:X} (${:X} != ${:X})", addr, data, expected);
		}
	}
}


#[cfg(test)]
mod test {
	use super::TestMemory;

	#[test]
	fn read () {
		let memory: TestMemory<u16> = TestMemory::new();
		assert_eq!(memory.get(0x0012), 0x12);
		assert_eq!(memory.get(0x0123), 0x24);
		assert_eq!(memory.get(0x1234), 0x46);
	}

	#[test]
	fn write () {
		let mut memory: TestMemory<u16> = TestMemory::new();
		memory.set(0x0012, 0x12);
		memory.set(0x0123, 0x24);
		memory.set(0x1234, 0x46);
	}
}
