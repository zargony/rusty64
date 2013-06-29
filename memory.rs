use std::num;
use addressable::Addressable;

struct Area<'self, ADDR> {
	base: ADDR,
	last: ADDR,
	handler: &'self mut Addressable<ADDR>,
}

pub struct Memory<'self, ADDR> {
	priv areas: ~[Area<'self, ADDR>],
}

impl<'self, ADDR> Memory<'self, ADDR> {
	pub fn new () -> Memory<ADDR> {
		Memory { areas: ~[] }
	}

	pub fn add<T: Send+Addressable<ADDR>> (&mut self, base: ADDR, last: ADDR, handler: &'self mut T) {
		self.areas.push(Area { base: base, last: last, handler: handler as &'self mut Addressable<ADDR> });
	}
}

impl<'self, ADDR: Int> Addressable<ADDR> for Memory<'self, ADDR> {
	pub fn get (&self, addr: ADDR) -> u8 {
		for self.areas.rev_iter().advance() |area| {
			if addr >= area.base && addr <= area.last {
				return area.handler.get(addr - area.base);
			}
		}
		fail!("memory: Read from unmapped memory area ($%x)", num::cast(addr));
	}

	pub fn set (&mut self, addr: ADDR, data: u8) {
		for self.areas.mut_rev_iter().advance() |area| {
			if addr >= area.base && addr <= area.last {
				area.handler.set(addr - area.base, data);
				return;
			}
		}
		fail!("memory: Write to unmapped memory area ($%x)", num::cast(addr));
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use addressable::Addressable;
	use ram::Ram;

	#[test]
	fn test_read_write () {
		let mut m: Memory<u16> = Memory::new();
		let mut ram1: Ram<u16> = Ram::new_sized(0x0400);
		let mut ram2: Ram<u16> = Ram::new_sized(0x0400);
		m.add(0x0000, 0x0400, &mut ram1);
		m.add(0x0200, 0x0600, &mut ram2);
		m.set(0x0300, 123);
		assert_eq!(m.get(0x0300), 123);
	}
}
