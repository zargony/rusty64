#[desc = "C64 emulator"];
#[copyright = "Andreas Neuhaus <info@zargony.com>"];
#[license = "MIT"];

extern mod native;

use cpu::Mos6510;
use mem::{Addressable, Ram, Rom, SharedMemory};

mod cpu;
mod mem;
mod ui;

#[start]
fn start (argc: int, argv: **u8) -> int {
	// Ensure that main is started on the native main os thread (required for SDL2)
	native::start(argc, argv, main)
}

fn main () {
	let mut c64 = C64::new();
	c64.run();
}


// General information on C64 : http://unusedino.de/ec64/technical/aay/c64/
// Useful emulator information: http://emudocs.org/?page=Commodore%2064
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
			kernal: Rom::new(&Path::new("c64/kernal.rom")),
			basic: Rom::new(&Path::new("c64/basic.rom")),
			characters: Rom::new(&Path::new("c64/characters.rom")),
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


pub struct C64 {
	priv cpu: Mos6510<SharedMemory<C64Memory>>,
	priv mem: SharedMemory<C64Memory>,
}

impl C64 {
	pub fn new () -> C64 {
		let mem = SharedMemory::new(C64Memory::new());
		C64 {
			cpu: Mos6510::new(mem.clone()),
			mem: mem,
		}
	}

	pub fn run (&mut self) {
		// TODO
	}
}
