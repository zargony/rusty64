//!
//! Memory types for testing
//!

use mem::Addressable;

/// Test-memory that returns/expects the lower nibble of the address as data. Reading the memory
/// always returns a data byte that equals the lower nibble of the requested address. Writing
/// the memory asserts that the set data byte equals the lower nibble of the requested address.
pub struct TestMemory;

impl TestMemory {
    pub fn new () -> TestMemory {
        TestMemory
    }
}

impl Addressable<u16> for TestMemory {
    fn get (&self, addr: u16) -> u8 {
        addr as u8
    }

    fn set (&mut self, addr: u16, data: u8) {
        assert_eq!(data, addr as u8);
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
