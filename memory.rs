use addressable::Addressable;

struct Memory<'self, ADDR> {
	getfn: &'self fn(ADDR) -> u8,
	setfn: &'self fn(ADDR, u8),
}

impl<'self, ADDR> Memory<'self, ADDR> {
	pub fn new (getfn: &'self fn(ADDR) -> u8, setfn: &'self fn(ADDR, u8)) -> Memory<'self, ADDR> {
		Memory { getfn: getfn, setfn: setfn }
	}
}

impl<'self, ADDR: Int> Addressable<ADDR> for Memory<'self, ADDR> {
	pub fn get (&self, addr: ADDR) -> u8 {
		(self.getfn)(addr)
	}

	pub fn set (&mut self, addr: ADDR, data: u8) {
		(self.setfn)(addr, data);
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use ram::Ram;

	#[test]
	fn test_get_set () {
		let mut ram: Ram<u16> = Ram::new_sized(0x0400);
		let mut mem = Memory::new(|addr| { ram.get(addr) }, |addr, data| { ram.set(addr, data); });
		mem.set(0x0200, 123);
		assert_eq!(mem.get(0x0200), 123);
		assert_eq!(ram.get(0x0200), 123);
		ram.set(0x0200, 234);
		assert_eq!(mem.get(0x0200), 234);
		assert_eq!(ram.get(0x0200), 234);
	}
}
