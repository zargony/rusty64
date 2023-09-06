//! Memory types for testing

use crate::addr::Address;
use crate::mem::Addressable;

/// Test-memory that returns/expects the sum of the lower and higher nibble of the address as data.
/// Reading the memory always returns a data byte that equals the sum of the lower and higher
/// nibble of the requested address. Writing the memory asserts that the set data byte equals the
/// sum of the lower and hight nibble of the requested address.
pub struct TestMemory;

impl TestMemory {
    pub fn new() -> TestMemory {
        TestMemory
    }

    /// Calculate the data byte for a given address
    fn addr2data<A: Address>(addr: A) -> u8 {
        let addr = addr.to_u16();
        (addr as u8).wrapping_add((addr >> 8) as u8)
    }
}

impl Addressable for TestMemory {
    fn get<A: Address>(&self, addr: A) -> u8 {
        TestMemory::addr2data(addr)
    }

    fn set<A: Address>(&mut self, addr: A, data: u8) {
        assert_eq!(data, TestMemory::addr2data(addr));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read() {
        let memory = TestMemory::new();
        assert_eq!(memory.get(0x0123), 0x24);
        assert_eq!(memory.get(0x1234), 0x46);
    }

    #[test]
    fn write() {
        let mut memory = TestMemory::new();
        memory.set(0x0123, 0x24);
        memory.set(0x1234, 0x46);
    }

    #[test]
    #[should_panic]
    fn write_fail() {
        let mut memory = TestMemory::new();
        memory.set(0x0123, 0x55);
    }
}
