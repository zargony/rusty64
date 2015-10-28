//!
//! Random Access Memory (RAM)
//!

use rand;
use addr::Address;
use mem::Addressable;

/// Generic read/write memory (RAM)
pub struct Ram {
    data: Vec<u8>,
    last_addr: u16,
}

impl Ram {
    /// Create new RAM with full capacity of its address range. The whole address space is filled
    /// with random bytes initially.
    pub fn new () -> Ram {
        Ram::with_capacity(!0)
    }

    /// Create new RAM which will be addressable from 0 to the given address. The whole address
    /// space is filled with random bytes initially.
    pub fn with_capacity (last_addr: u16) -> Ram {
        let data = 0.successive().upto(last_addr).map(|_| rand::random()).collect();
        Ram { data: data, last_addr: last_addr }
    }

    /// Returns the capacity of the RAM
    pub fn capacity (&self) -> usize {
        self.data.len()
    }
}

impl Addressable for Ram {
    fn get<A: Address> (&self, addr: A) -> u8 {
        if addr.to_u16() > self.last_addr {
            panic!("ram: Read beyond memory bounds ({} > {})", addr.display(), self.last_addr.display());
        }
        self.data[addr.to_u16() as usize]
    }

    fn set<A: Address> (&mut self, addr: A, data: u8) {
        if addr.to_u16() > self.last_addr {
            panic!("ram: Write beyond memory bounds ({} > {})", addr.display(), self.last_addr.display());
        }
        self.data[addr.to_u16() as usize] = data;
    }
}


#[cfg(test)]
mod tests {
    use mem::Addressable;
    use super::*;

    #[test]
    fn create_with_full_addressable_capacity () {
        let memory = Ram::new();
        assert_eq!(memory.capacity(), 65536);
    }

    #[test]
    fn create_with_requested_capacity () {
        let memory = Ram::with_capacity(0x03ff);
        assert_eq!(memory.capacity(), 1024);
    }

    #[test]
    fn read_write () {
        let mut memory = Ram::with_capacity(0x03ff_u16);
        memory.set(0x0123, 0x55);
        assert_eq!(memory.get(0x0123), 0x55);
    }
}
