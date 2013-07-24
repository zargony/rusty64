use addressable::Addressable;
use ram::Ram;
use rom::Rom;

// C64 memory map overview: http://www.c64-wiki.com/index.php/Memory_Map
// Details about the PLA: http://www.c64-wiki.de/index.php/PLA_(C64-Chip)
// Even more PLA details: http://skoe.de/docs/c64-dissected/pla/c64_pla_dissected_r1.1_a4ss.pdf

// 64K RAM (not every part is always accessible)
// 8K BASIC ROM at $A000
// 4K character ROM at $D000
// 8K kernal ROM at $E000

struct C64Memory {
	ram: Ram<u16>,
	kernal: Rom<u16>,
	basic: Rom<u16>,
	characters: Rom<u16>,
}

impl C64Memory {
	fn new () -> C64Memory {
		C64Memory {
			ram: Ram::new(),
			kernal: Rom::new(&Path("kernal.rom")),
			basic: Rom::new(&Path("basic.rom")),
			characters: Rom::new(&Path("characters.rom")),
		}
	}
}

impl Addressable<u16> for C64Memory {
	fn get (&self, addr: u16) -> u8 {
		match addr {
			// TODO: Switch memory access based on LORAM/HIRAM/CHAREN
			0xa000..0xbfff => self.basic.get(addr - 0xa000),
			0xd000..0xdfff => self.characters.get(addr - 0xd000),
			0xe000..0xffff => self.kernal.get(addr - 0xe000),
			_              => self.ram.get(addr),
		}
	}

	fn set (&mut self, addr: u16, data: u8) {
		self.ram.set(addr, data);
	}
}


pub struct C64;

impl C64 {
	pub fn new () -> C64 {
		C64
	}

	pub fn run (&mut self) {
		// TODO
	}
}
