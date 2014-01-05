use std::cell::RefCell;
use std::rc::Rc;
use super::{Addr, Addressable};

/// Shared memory. Allows some arbitrary memory to be shared by cloning.
pub struct SharedMemory<M> {
	priv mem: Rc<RefCell<M>>,
}

impl<A: Addr, M: Addressable<A> + Freeze> SharedMemory<M> {
	/// Create new shared memory of the given memory object
	pub fn new (mem: M) -> SharedMemory<M> {
		SharedMemory { mem: Rc::from_mut(RefCell::new(mem)) }
	}
}

impl<A: Addr, M: Addressable<A>> Addressable<A> for SharedMemory<M> {
	fn get (&self, addr: A) -> u8 {
		self.mem.borrow().with(|mem| mem.get(addr.clone()))
	}

	fn set (&mut self, addr: A, data: u8) {
		self.mem.borrow().with_mut(|mem| mem.set(addr.clone(), data));
	}
}

impl<A: Addr, M: Addressable<A>> Clone for SharedMemory<M> {
	fn clone (&self) -> SharedMemory<M> {
		SharedMemory { mem: self.mem.clone() }
	}
}


#[cfg(test)]
mod test {
	use super::super::Addressable;
	use super::SharedMemory;

	struct TestMemory {
		mem: [u8, ..256],
	}
	impl Addressable<u8> for TestMemory {
		fn get (&self, addr: u8) -> u8 { self.mem[addr] }
		fn set (&mut self, addr: u8, data: u8) { self.mem[addr] = data; }
	}

	#[test]
	fn read_write () {
		let mut mem = TestMemory { mem: [0, ..256] };
		mem.set(0x12, 0x34);
		let mut shmem = SharedMemory::new(mem);
		assert_eq!(shmem.get(0x12), 0x34);
		shmem.set(0x56, 0x78);
		assert_eq!(shmem.get(0x56), 0x78);
	}

	#[test]
	fn read_write_cloned () {
		let mut mem = TestMemory { mem: [0, ..256] };
		mem.set(0x12, 0x34);
		let mut shmem1 = SharedMemory::new(mem);
		let mut shmem2 = shmem1.clone();
		assert_eq!(shmem2.get(0x12), 0x34);
		shmem1.set(0x56, 0x78);
		assert_eq!(shmem2.get(0x56), 0x78);
		shmem2.set(0x9a, 0xbc);
		assert_eq!(shmem1.get(0x9a), 0xbc);
	}
}
