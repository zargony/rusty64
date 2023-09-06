//! Generic implementations for shared (wrapped) addressable objects

use super::Addressable;
use crate::addr::Address;
use std::cell::RefCell;
use std::rc::Rc;

impl<M: Addressable> Addressable for RefCell<M> {
    fn get<A: Address>(&self, addr: A) -> u8 {
        self.borrow().get(addr)
    }

    fn set<A: Address>(&mut self, addr: A, data: u8) {
        self.borrow_mut().set(addr, data)
    }
}

impl<M: Addressable> Addressable for Rc<RefCell<M>> {
    fn get<A: Address>(&self, addr: A) -> u8 {
        (**self).borrow().get(addr)
    }

    fn set<A: Address>(&mut self, addr: A, data: u8) {
        (**self).borrow_mut().set(addr, data)
    }
}

#[cfg(test)]
mod tests {
    use super::super::Ram;
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn read_write() {
        let mut mem = Rc::new(RefCell::new(Ram::new()));
        mem.set(0x12, 0x34);
        assert_eq!(mem.get(0x12), 0x34);
        mem.set(0x56, 0x78);
        assert_eq!(mem.get(0x56), 0x78);
    }

    #[test]
    fn read_write_shared() {
        let mut mem1 = Rc::new(RefCell::new(Ram::new()));
        mem1.set(0x12, 0x34);
        let mut mem2 = mem1.clone();
        assert_eq!(mem2.get(0x12), 0x34);
        mem1.set(0x56, 0x78);
        assert_eq!(mem2.get(0x56), 0x78);
        mem2.set(0x9a, 0xbc);
        assert_eq!(mem1.get(0x9a), 0xbc);
    }
}
