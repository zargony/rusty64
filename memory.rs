use std::num;
use addressable::Addressable;

struct Area<ADDR> {
	base: ADDR,
	last: ADDR,
	handler: ~Addressable<ADDR>,
}

pub struct Memory<ADDR> {
	priv areas: ~[Area<ADDR>],
}

impl<ADDR> Memory<ADDR> {
	pub fn new () -> Memory<ADDR> {
		Memory { areas: ~[] }
	}

	pub fn add<T: Send+Addressable<ADDR>> (&mut self, base: ADDR, last: ADDR, handler: ~T) {
		self.areas.push(Area { base: base, last: last, handler: handler as ~Addressable<ADDR> });
	}
}

impl<ADDR: Int> Addressable<ADDR> for Memory<ADDR> {
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
		let ram1: Ram<u16> = Ram::new_sized(0x0400);
		let ram2: Ram<u16> = Ram::new_sized(0x0400);
		m.add(0x0000, 0x0400, ~ram1);
		m.add(0x0200, 0x0600, ~ram2);
		m.set(0x0300, 123);
		assert_eq!(m.get(0x0300), 123);
	}
}
