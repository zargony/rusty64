use mem::Addressable;
use super::cpu::CPU;

// General information on 65xx: http://en.wikipedia.org/wiki/MOS_Technology_6510
// Useful emulator information: http://emudocs.org/?page=CPU%2065xx
// Web simulator and much info: http://e-tradition.net/bytes/6502/
// Good reference and overview: http://www.obelisk.demon.co.uk/index.html
// Processor bugs and caveats : http://www.textfiles.com/apple/6502.bugs.txt

/// Hard-coded address where to look for the address to jump to on nonmaskable interrupt
static NMI_VECTOR:			u16 = 0xfffa;
/// Hard-coded address where to look for the address to jump to on reset
static RESET_VECTOR:		u16 = 0xfffc;
/// Hard-coded address where to look for the address to jump to on interrupt
static IRQ_VECTOR:			u16 = 0xfffe;

/// The MOS6502 processor
pub struct Mos6502 {
	priv pc: u16,						// Program Counter
	priv ac: u8,						// Accumulator
	priv x: u8,							// X register
	priv y: u8,							// Y register
	priv sr: u8,						// Status Register (NV-BDIZC: Negative, oVerflow, 1, Break, Decimal, Interrupt, Zero, Carry)
	priv sp: u8,						// Stack Pointer
}

impl Mos6502 {
	/// Create a new MOS6502 processor
	pub fn new () -> Mos6502 {
		Mos6502 { pc: 0, ac: 0, x: 0, y: 0, sr: 0, sp: 0 }
	}

	/// Get the memory contents at the current PC and advance the PC
	fn next<M: Addressable<u16>, T: Primitive> (&mut self, mem: &M) -> T {
		let val = mem.get_le(self.pc);
		self.pc += Primitive::bytes(None::<T>) as u16;
		val
	}
}

impl CPU<u16> for Mos6502 {
	/// Reset the CPU
	fn reset<M: Addressable<u16>> (&mut self, mem: &M) {
		// On reset, the interrupt-disable flag is set (and the decimal flag is cleared in the CMOS version 65c02).
		// The other bits and all registers (including the stack pointer are unspecified and might contain random values.
		// Execution begins at the address pointed to by the reset vector at address $FFFC.
		self.pc = mem.get_le(RESET_VECTOR);
		self.sr = 0x24;
		debug!("mos65xx: Reset! Start at (${:04X}) -> ${:04X}", RESET_VECTOR, self.pc);
	}

	/// Do one step (execute the next instruction). Returns the number of
	/// cycles the instruction needed
	fn step<M: Addressable<u16>> (&mut self, mem: &mut M) -> uint {
		// TODO...
		0
	}
}


#[cfg(test)]
mod test {
	use mem::test::TestMemory;
	use super::Mos6502;

	#[test]
	fn initial_state () {
		let cpu = Mos6502::new();
		assert_eq!(cpu.pc, 0x0000);
	}

	#[test]
	fn state_after_reset () {
		let mut cpu = Mos6502::new();
		let mem: TestMemory<u16> = TestMemory::new();
		cpu.reset(&mem);
		assert_eq!(cpu.pc, 0xfcfb);
		assert_eq!(cpu.sr, 0x24);
	}

	#[test]
	fn fetch_memory_contents_and_advance_pc () {
		let mut cpu = Mos6502::new();
		let mem: TestMemory<u16> = TestMemory::new();
		let val: u8 = cpu.next(&mem); assert_eq!(val, 0x00);
		let val: u8 = cpu.next(&mem); assert_eq!(val, 0x01);
		let val: u16 = cpu.next(&mem); assert_eq!(val, 0x0302);
		let val: u16 = cpu.next(&mem); assert_eq!(val, 0x0504);
	}
}
