//!
//! Memory types for testing
//!

use addr::Address;
use mem::Addressable;

/// Test-memory that returns/expects the lower nibble of the address as data. Reading the memory
/// always returns a data byte that equals the lower nibble of the requested address. Writing
/// the memory asserts that the set data byte equals the lower nibble of the requested address.
pub struct TestMemory;

impl TestMemory {
    pub fn new () -> TestMemory {
        TestMemory
    }

    /// Calculate the data byte for a given address
    fn addr2data<A: Address> (addr: A) -> u8 {
        unsafe { addr.to_usize() as u8 }
    }
}

impl Addressable for TestMemory {
    fn get<A: Address> (&self, addr: A) -> u8 {
        TestMemory::addr2data(addr)
    }

    fn set<A: Address> (&mut self, addr: A, data: u8) {
        assert_eq!(data, TestMemory::addr2data(addr));
    }
}


#[cfg(test)]
mod tests {
    use mem::Addressable;
    use super::*;

    #[test]
    fn read () {
        let memory = TestMemory::new();
        assert_eq!(memory.get(0x0123), 0x23);
        assert_eq!(memory.get(0x1234), 0x34);
    }

    #[test]
    fn write () {
        let mut memory = TestMemory::new();
        memory.set(0x0123, 0x23);
        memory.set(0x1234, 0x34);
    }

    #[test] #[should_panic]
    fn write_fail () {
        let mut memory = TestMemory::new();
        memory.set(0x0123, 0x55);
    }
}
