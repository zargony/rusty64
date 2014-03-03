#[desc = "C64 emulator"];
#[copyright = "Andreas Neuhaus <info@zargony.com>"];
#[license = "MIT"];

extern crate native;

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

/// Represents the memory as seen by the CPU (memory layout is based on the
/// settings of bits 2..0 of the processor port (/CHAREN, /HIRAM and /LORAM)
struct CpuMemory {
	ram: SharedMemory<Ram<u16>>,
	basic: Rom<u16>,
	characters: Rom<u16>,
	kernal: Rom<u16>,
	charen: bool,
	hiram: bool,
	loram: bool,
}

impl CpuMemory {
	/// Create a new CPU memory
	fn new () -> CpuMemory {
		CpuMemory {
			ram: SharedMemory::new(Ram::new()),
			basic: Rom::new(&Path::new("c64/basic.rom")),
			characters: Rom::new(&Path::new("c64/characters.rom")),
			kernal: Rom::new(&Path::new("c64/kernal.rom")),
			charen: false,
			hiram: false,
			loram: false,
		}
	}
}

impl Addressable<u16> for CpuMemory {
	fn get (&self, addr: u16) -> u8 {
		// Switch memory access based on processor port lines:
		//
		//    /CHAREN /HIRAM /LORAM $A000-$BFFF $D000-$DFFF $E000-$FFFF
		// #1    1      1       1      BASIC        I/O        KERNAL
		// #6    1      1       0       RAM         I/O        KERNAL
		// #3    1      0       1       RAM         I/O         RAM
		// #5    X      0       0       RAM         RAM         RAM
		// #2    0      1       1      BASIC       CHARS       KERNAL
		// #7    0      1       0       RAM        CHARS       KERNAL
		// #4    0      0       1       RAM        CHARS        RAM
		// #5    X      0       0       RAM         RAM         RAM
		//
		// TODO: Hardware cartridges can as well use /GAME and /EXROM
		match (addr, self.charen, self.hiram, self.loram) {
			(0xa000..0xbfff,     _, false, false) => self.basic.get(addr - 0xa000),				// #1,2
			(0xd000..0xdfff, false, false,     _) |												// #1,6
			(0xd000..0xdfff, false,  true, false) => 0, // TODO: self.io.get(addr - 0xd000),	// #3
			(0xd000..0xdfff,  true, false,     _) |												// #2,7
			(0xd000..0xdfff,  true,  true, false) => self.characters.get(addr - 0xd000),		// #4
			(0xe000..0xffff,     _, false,     _) => self.kernal.get(addr - 0xe000),			// #1,6,2,7
			_                                     => self.ram.get(addr),
		}
	}

	fn set (&mut self, addr: u16, data: u8) {
		// Writing to an address will always store the data to RAM,
		// no matter if an address is accessed that is mapped to ROM
		match (addr, self.charen, self.hiram, self.loram) {
			(0xd000..0xdfff, false, false,     _) |												// #1,6
			(0xd000..0xdfff, false,  true, false) => (), // self.io.set(addr - 0xd000, data),	// #3
			_                                     => self.ram.set(addr, data),
		}
	}
}


pub struct C64 {
	priv cpu: Mos6510<CpuMemory>,
}

impl C64 {
	/// Create a new C64 emulator
	pub fn new () -> C64 {
		let mem = CpuMemory::new();
		C64 {
			cpu: Mos6510::new(mem),
		}
	}

	/// Run the C64 emulation
	pub fn run (&mut self) {
		// TODO
	}
}
