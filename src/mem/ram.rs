//!
//! Random Access Memory (RAM)
//!

use rand;
use super::{Address, Addressable};

/// Generic read/write memory (RAM)
pub struct Ram<A> {
    data: Vec<u8>,
    last_addr: A,
}

impl<A: Address> Ram<A> {
    /// Create new RAM with full capacity of its address range. The whole address space is filled
    /// with random bytes initially.
    pub fn new () -> Ram<A> {
        Ram::with_capacity(!A::zero())
    }

    /// Create new RAM which will be addressable from 0 to the given address. The whole address
    /// space is filled with random bytes initially.
    pub fn with_capacity (last_addr: A) -> Ram<A> {
        let data = A::zero().upto(last_addr).map(|_| rand::random()).collect();
        Ram { data: data, last_addr: last_addr }
    }

    /// Returns the capacity of the RAM
    #[allow(dead_code)]
    pub fn capacity (&self) -> usize {
        self.data.len()
    }
}

impl<A: Address> Addressable<A> for Ram<A> {
    fn get (&self, addr: A) -> u8 {
        if addr > self.last_addr {
            panic!("ram: Read beyond memory bounds ({} > {})", addr.display(), self.last_addr.display());
        }
        unsafe { self.data[addr.to_usize()] }
    }

    fn set (&mut self, addr: A, data: u8) {
        if addr > self.last_addr {
            panic!("ram: Write beyond memory bounds ({} > {})", addr.display(), self.last_addr.display());
        }
        unsafe { self.data[addr.to_usize()] = data; }
    }
}


#[cfg(test)]
mod tests {
    use super::super::Addressable;
    use super::*;

    #[test]
    fn create_with_full_addressable_capacity () {
        let memory: Ram<u8> = Ram::new();
        assert_eq!(memory.capacity(), 256);
    }

    #[test]
    fn create_with_requested_capacity () {
        let memory: Ram<u16> = Ram::with_capacity(0x03ff);
        assert_eq!(memory.capacity(), 1024);
    }

    #[test]
    fn read_write () {
        let mut memory: Ram<u16> = Ram::with_capacity(0x03ff_u16);
        memory.set(0x0123, 0x55);
        assert_eq!(memory.get(0x0123), 0x55);
    }
}
