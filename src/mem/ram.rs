use rand;
use std::iter;
use std::num::{self, Int};
use super::{Address, Addressable};

/// Generic read/write memory (RAM)
pub struct Ram<A> {
    data: Vec<u8>,
    last_addr: A,
}

impl<A: Address> Ram<A> {
    /// Create new RAM with full capacity of its address range. The whole
    /// address space is filled with random bytes initially.
    pub fn new () -> Ram<A> {
        Ram::with_capacity(Int::max_value())
    }

    /// Create new RAM which will be addressable from 0 to the given address.
    /// The whole address space is filled with random bytes initially.
    pub fn with_capacity (last_addr: A) -> Ram<A> {
        // FIXME: Use range notation instead of calling iter::range_inclusive
        let data: Vec<u8> = iter::range_inclusive(Int::zero(), last_addr).map(|_| rand::random()).collect();
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
        let i: usize = num::cast(addr).unwrap();
        self.data[i]
    }

    fn set (&mut self, addr: A, data: u8) {
        if addr > self.last_addr {
            panic!("ram: Write beyond memory bounds ({} > {})", addr.display(), self.last_addr.display());
        }
        let i: usize = num::cast(addr).unwrap();
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
        assert_eq!(memory.capacity(), 256);
    }

    #[test]
    fn create_with_requested_capacity () {
        let memory: Ram<u16> = Ram::with_capacity(0x03ff_u16);
        assert_eq!(memory.capacity(), 1024);
    }

    #[test]
    fn read_write () {
        let mut memory: Ram<u16> = Ram::with_capacity(0x03ff_u16);
        memory.set(0x0123, 0x55);
        assert_eq!(memory.get(0x0123), 0x55);
    }
}
