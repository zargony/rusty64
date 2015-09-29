use std::cell::RefCell;
use std::rc::Rc;
use super::{Address, Addressable};

impl<A: Address, M: Addressable<A>> Addressable<A> for RefCell<M> {
    fn get (&self, addr: A) -> u8 { self.borrow().get(addr) }
    fn set (&mut self, addr: A, data: u8) { self.borrow_mut().set(addr, data) }
}

impl<A: Address, M: Addressable<A>> Addressable<A> for Rc<RefCell<M>> {
    fn get (&self, addr: A) -> u8 { (**self).borrow().get(addr) }
    fn set (&mut self, addr: A, data: u8) { (**self).borrow_mut().set(addr, data) }
}


#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use super::super::Addressable;

    struct TestMemory {
        mem: [u8; 256],
    }
    impl Addressable<u8> for TestMemory {
        fn get (&self, addr: u8) -> u8 { self.mem[addr as usize] }
        fn set (&mut self, addr: u8, data: u8) { self.mem[addr as usize] = data; }
    }

    #[test]
    fn read_write () {
        let mut mem = Rc::new(RefCell::new(TestMemory { mem: [0; 256] }));
        mem.set(0x12, 0x34);
        assert_eq!(mem.get(0x12), 0x34);
        mem.set(0x56, 0x78);
        assert_eq!(mem.get(0x56), 0x78);
    }

    #[test]
    fn read_write_shared () {
        let mut mem1 = Rc::new(RefCell::new(TestMemory { mem: [0; 256] }));
        mem1.set(0x12, 0x34);
        let mut mem2 = mem1.clone();
        assert_eq!(mem2.get(0x12), 0x34);
        mem1.set(0x56, 0x78);
        assert_eq!(mem2.get(0x56), 0x78);
        mem2.set(0x9a, 0xbc);
        assert_eq!(mem1.get(0x9a), 0xbc);
    }
}
