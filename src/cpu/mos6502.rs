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

/// The MOS6502 status flags
pub enum StatusFlag {
	CarryFlag				= 0,
	ZeroFlag				= 1,
	InterruptDisableFlag	= 2,
	DecimalFlag				= 3,
	BreakFlag				= 4,
	OverflowFlag			= 6,
	NegativeFlag			= 7,
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

	/// Get the given status flag
	fn get_flag (&self, flag: StatusFlag) -> bool {
		(self.sr & (1 << flag as u8)) != 0
	}

	/// Set the given status flag to the given state
	fn set_flag (&mut self, flag: StatusFlag, state: bool) {
		if state {
			self.sr |= (1 << flag as u8);
		} else {
			self.sr &= !(1 << flag as u8);
		}
	}

	/// Set ZeroFlag and NegativeFlag based on the given value
	fn set_zn (&mut self, value: u8) -> u8 {
		self.set_flag(ZeroFlag, value == 0);
		self.set_flag(NegativeFlag, (value as i8) < 0);
		value
	}

	/// Push a value onto the stack
	fn push<M: Addressable<u16>, T: Primitive> (&mut self, mem: &mut M, value: T) {
		// SP points to the next free stack position as $0100+SP. SP needs to be
		// initialized to #$FF by the reset code. As the stack grows, SP decreases
		// down to #$00 (i.e. stack full). Stack access never leaves the stack page!
		self.sp -= Primitive::bytes(None::<T>) as u8;
		mem.set_le_masked(0x0100 + (self.sp + 1) as u16, 0x00ff, value);
	}

	/// Pop a value from the stack
	fn pop<M: Addressable<u16>, T: Primitive> (&mut self, mem: &M) -> T {
		// See push() for details
		let value: T = mem.get_le_masked(0x0100 + (self.sp + 1) as u16, 0x00ff);
		self.sp += Primitive::bytes(None::<T>) as u8;
		value
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
	use mem::{Addressable, Ram};
	use super::Mos6502;
	use super::{CarryFlag, ZeroFlag, OverflowFlag, NegativeFlag};

	struct TestMemory;
	impl Addressable<u16> for TestMemory {
		fn get (&self, addr: u16) -> u8 { addr as u8 }
		fn set (&mut self, addr: u16, data: u8) { assert_eq!(data, addr as u8); }
	}

	#[test]
	fn initial_state () {
		let cpu = Mos6502::new();
		assert_eq!(cpu.pc, 0x0000);
	}

	#[test]
	fn state_after_reset () {
		let mut cpu = Mos6502::new();
		let mem = TestMemory;
		cpu.reset(&mem);
		assert_eq!(cpu.pc, 0xfdfc);
		assert_eq!(cpu.sr, 0x24);
	}

	#[test]
	fn fetch_memory_contents_and_advance_pc () {
		let mut cpu = Mos6502::new();
		let mem = TestMemory;
		cpu.pc = 0x12;
		let val: u8 = cpu.next(&mem); assert_eq!(val, 0x12);
		let val: u8 = cpu.next(&mem); assert_eq!(val, 0x13);
		let val: u16 = cpu.next(&mem); assert_eq!(val, 0x1514);
		let val: u16 = cpu.next(&mem); assert_eq!(val, 0x1716);
	}

	#[test]
	fn status_flags () {
		let mut cpu = Mos6502::new();
		cpu.sr = 0xaa;
		assert!(!cpu.get_flag(CarryFlag));
		assert!(cpu.get_flag(ZeroFlag));
		assert!(!cpu.get_flag(OverflowFlag));
		assert!(cpu.get_flag(NegativeFlag));
		cpu.set_flag(CarryFlag, true);
		cpu.set_flag(ZeroFlag, false);
		cpu.set_flag(OverflowFlag, true);
		cpu.set_flag(NegativeFlag, false);
		assert_eq!(cpu.sr, 0x69);
	}

	#[test]
	fn zero_and_negative_values () {
		let mut cpu = Mos6502::new();
		cpu.set_zn(0);
		assert!(cpu.get_flag(ZeroFlag));
		assert!(!cpu.get_flag(NegativeFlag));
		cpu.set_zn(42);
		assert!(!cpu.get_flag(ZeroFlag));
		assert!(!cpu.get_flag(NegativeFlag));
		cpu.set_zn(142);
		assert!(!cpu.get_flag(ZeroFlag));
		assert!(cpu.get_flag(NegativeFlag));
	}

	#[test]
	fn stack_push_pop () {
		let mut cpu = Mos6502::new();
		let mut mem: Ram<u16> = Ram::with_capacity(0x01ff_u16);
		cpu.sp = 0xff;
		assert_eq!(mem.get(0x01ff), 0x00);
		cpu.push(&mut mem, 0x12_u8);
		assert_eq!(cpu.sp, 0xfe);
		assert_eq!(mem.get(0x01ff), 0x12);
		assert_eq!(mem.get(0x01fe), 0x00);
		cpu.push(&mut mem, 0x3456_u16);
		assert_eq!(cpu.sp, 0xfc);
		assert_eq!(mem.get(0x01fe), 0x34);
		assert_eq!(mem.get(0x01fd), 0x56);
		assert_eq!(mem.get(0x01fc), 0x00);
		let val: u8 = cpu.pop(&mem);
		assert_eq!(val, 0x56);
		assert_eq!(cpu.sp, 0xfd);
		let val: u16 = cpu.pop(&mem);
		assert_eq!(val, 0x1234);
		assert_eq!(cpu.sp, 0xff);
	}

	#[test]
	fn stack_overflow () {
		let mut cpu = Mos6502::new();
		let mut mem: Ram<u16> = Ram::with_capacity(0x01ff_u16);
		cpu.sp = 0x00;
		cpu.push(&mut mem, 0x12_u8);
		assert_eq!(cpu.sp, 0xff);
		assert_eq!(mem.get(0x0100), 0x12);
		let val: u8 = cpu.pop(&mem);
		assert_eq!(val, 0x12);
		assert_eq!(cpu.sp, 0x00);
	}

	#[test]
	fn stack_overflow_word () {
		let mut cpu = Mos6502::new();
		let mut mem: Ram<u16> = Ram::with_capacity(0x01ff_u16);
		cpu.sp = 0x00;
		cpu.push(&mut mem, 0x1234_u16);
		assert_eq!(cpu.sp, 0xfe);
		assert_eq!(mem.get(0x0100), 0x12);
		assert_eq!(mem.get(0x01ff), 0x34);
		let val: u16 = cpu.pop(&mem);
		assert_eq!(val, 0x1234);
		assert_eq!(cpu.sp, 0x00);
	}
}
