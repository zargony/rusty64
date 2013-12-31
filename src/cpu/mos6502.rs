use mem::Addressable;
use super::cpu::CPU;

// General information on 65xx: http://en.wikipedia.org/wiki/MOS_Technology_6510
// Useful emulator information: http://emudocs.org/?page=CPU%2065xx
// Web simulator and much info: http://e-tradition.net/bytes/6502/
// Good reference and overview: http://www.obelisk.demon.co.uk/index.html
// Processor bugs and caveats : http://www.textfiles.com/apple/6502.bugs.txt

/// Processor instructions
enum Instruction {
	// Load/store operations
	LDA, LDX, LDY, STA, STX, STY,
	// Register transfers
	TAX, TAY, TXA, TYA,
	// Stack operations
	TSX, TXS, PHA, PHP, PLA, PLP,
	// Logical
	AND, EOR, ORA, BIT,
	// Arithmetic
	ADC, SBC, CMP, CPX, CPY,
	// Increments & decrements
	INC, INX, INY, DEC, DEX, DEY,
	// Shifts
	ASL, LSR, ROL, ROR,
	// Jump & calls
	JMP, JSR, RTS,
	// Branches
	BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS,
	// Status flag changes
	CLC, CLD, CLI, CLV, SEC, SED, SEI,
	// System functions
	BRK, NOP, RTI,
}

impl Instruction {
	/// Returns a printable instruction mnemonic
	fn as_str (&self) -> &'static str {
		match *self {
			LDA => "LDA", LDX => "LDX", LDY => "LDY", STA => "STA", STX => "STX", STY => "STY",
			TAX => "TAX", TAY => "TAY", TXA => "TXA", TYA => "TYA",
			TSX => "TSX", TXS => "TXS", PHA => "PHA", PHP => "PHP", PLA => "PLA", PLP => "PLP",
			AND => "AND", EOR => "EOR", ORA => "ORA", BIT => "BIT",
			ADC => "ADC", SBC => "SBC", CMP => "CMP", CPX => "CPX", CPY => "CPY",
			INC => "INC", INX => "INX", INY => "INY", DEC => "DEC", DEX => "DEX", DEY => "DEY",
			ASL => "ASL", LSR => "LSR", ROL => "ROL", ROR => "ROR",
			JMP => "JMP", JSR => "JSR", RTS => "RTS",
			BCC => "BCC", BCS => "BCS", BEQ => "BEQ", BMI => "BMI", BNE => "BNE", BPL => "BPL", BVC => "BVC", BVS => "BVS",
			CLC => "CLC", CLD => "CLD", CLI => "CLI", CLV => "CLV", SEC => "SEC", SED => "SED", SEI => "SEI",
			BRK => "BRK", NOP => "NOP", RTI => "RTI",
		}
	}
}

/// Instruction operand with different addressing modes
enum Operand {
	Implied,							// OPC				operand implied
	Immediate(u8),						// OPC #$BB			operand is value $BB
	Accumulator,						// OPC A			operand is AC
	Relative(i8),						// OPC $RR			branch target is PC + signed offset $RR (bit 7 signifies negative offset)
	Absolute(u16),						// OPC $HHLL		operand is address $HHLL
	AbsoluteIndexedWithX(u16),			// OPC $HHLL,X		operand is address $HHLL incremented by X
	AbsoluteIndexedWithY(u16),			// OPC $HHLL,Y		operand is address $HHLL incremented by Y
	Indirect(u16),						// OPC ($HHLL)		operand is effective address; effective address is value of address; no page transition (MSB-bug)
	ZeroPage(u8),						// OPC $LL			operand is address $00LL
	ZeroPageIndexedWithX(u8),			// OPC $LL,X		operand is address $00LL incremented by X; no page transition
	ZeroPageIndexedWithY(u8),			// OPC $LL,Y		operand is address $00LL incremented by Y; no page transition
	ZeroPageIndexedWithXIndirect(u8),	// OPC ($LL,X)		operand is effective address; effective address is $00LL incremented by X; no page transition
	ZeroPageIndirectIndexedWithY(u8),	// OPC ($LL),Y		operand is effective address incremented by Y; effective address is word at $00LL
}

impl Operand {
	/// Returns a printable operand mnemonic
	fn as_str (&self) -> ~str {
		match *self {
			Implied => ~"",
			Immediate(value) => format!("\\#${:02X}", value),
			Accumulator => ~"A",
			Relative(offset) => format!("{:+d}", offset),
			Absolute(addr) => format!("${:04X}", addr),
			AbsoluteIndexedWithX(addr) => format!("${:04X},X", addr),
			AbsoluteIndexedWithY(addr) => format!("${:04X},Y", addr),
			Indirect(addr) => format!("(${:04X})", addr),
			ZeroPage(addr) => format!("${:02X}", addr),
			ZeroPageIndexedWithX(addr) => format!("${:02X},X", addr),
			ZeroPageIndexedWithY(addr) => format!("${:02X},Y", addr),
			ZeroPageIndexedWithXIndirect(addr) => format!("(${:02X},X)", addr),
			ZeroPageIndirectIndexedWithY(addr) => format!("(${:02X}),Y", addr),
		}
	}
}

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
	priv reset: bool,					// RESET line
	priv nmi: bool,						// NMI line
	priv irq: bool,						// IRQ line
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
		Mos6502 { pc: 0, ac: 0, x: 0, y: 0, sr: 0x20, sp: 0, reset: true, nmi: false, irq: false }
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

	/// Interrupt the CPU (NMI)
	pub fn nmi (&mut self) {
		// Trigger the NMI line. The actual NMI processing is done in the next step().
		self.nmi = true;
	}

	/// Interrupt the CPU (IRQ)
	pub fn irq (&mut self) {
		// Trigger the IRQ line. The actual IRQ processing is done in the next step().
		self.irq = true;
	}
}

impl CPU<u16> for Mos6502 {
	/// Reset the CPU
	fn reset (&mut self) {
		// Trigger the RESET line. The actual RESET processing is done in the next step().
		self.reset = true;
	}

	/// Do one step (execute the next instruction). Returns the number of
	/// cycles the instruction needed
	fn step<M: Addressable<u16>> (&mut self, mem: &mut M) -> uint {
		// Process RESET if line was triggered
		if self.reset {
			// A RESET jumps to the vector at RESET_VECTOR and sets the InterruptDisableFlag.
			// Note that all other states and registers are unspecified and might contain
			// random values, so they need to be initialized by the reset routine.
			// See also http://6502.org/tutorials/interrupts.html
			self.set_flag(InterruptDisableFlag, true);
			self.pc = mem.get_le(RESET_VECTOR);
			self.reset = false;
			self.nmi = false;
			self.irq = false;
			debug!("mos6502: RESET - Jumping to (${:04X}) -> ${:04X}", RESET_VECTOR, self.pc);
			return 6;
		}
		// Process NMI if line was triggered
		if self.nmi {
			// An NMI pushes PC and SR to the stack and jumps to the vector at NMI_VECTOR.
			// It does NOT set the InterruptDisableFlag.
			// See also http://6502.org/tutorials/interrupts.html
			self.push(mem, self.pc);
			self.push(mem, self.sr);
			self.pc = mem.get_le(NMI_VECTOR);
			self.nmi = false;
			debug!("mos6502: NMI - Jumping to (${:04X}) -> ${:04X}", NMI_VECTOR, self.pc);
			return 7;
		}
		// Process IRQ if line was triggered and interrupts are enabled
		if self.irq && !self.get_flag(InterruptDisableFlag) {
			// An IRQ pushes PC and SR to the stack, jumps to the vector at IRQ_VECTOR and
			// sets the InterruptDisableFlag.
			// The BRK instruction does the same, but sets BreakFlag (before pushing SR).
			// See also http://6502.org/tutorials/interrupts.html
			self.set_flag(BreakFlag, false);
			self.push(mem, self.pc);
			self.push(mem, self.sr);
			self.set_flag(InterruptDisableFlag, true);
			self.pc = mem.get_le(IRQ_VECTOR);
			// FIXME: The real 6502 IRQ line is level-sensitive, not edge-sensitive!
			// FIXME: I.e. it does not stop jumping to the IRQ_VECTOR after one run,
			// FIXME: but after the hardware drops the IRQ line (which the interrupt
			// FIXME: code usually causes, but not necessary needs to cause).
			self.irq = false;
			debug!("mos6502: IRQ - Jumping to (${:04X}) -> ${:04X}", IRQ_VECTOR, self.pc);
			return 7;
		}

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
		assert_eq!(cpu.sr, 0x20);
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

	#[test]
	fn state_after_reset () {
		let mut cpu = Mos6502::new();
		let mut mem: Ram<u16> = Ram::new();
		cpu.step(&mut mem);
		assert!(!cpu.reset && !cpu.nmi && !cpu.irq);
		cpu.sr = 0x23;
		cpu.sp = 0xff;
		mem.set_le(0xfffc, 0x1234);
		cpu.reset();
		cpu.step(&mut mem);
		assert_eq!(cpu.pc, 0x1234);
		assert_eq!(cpu.sr, 0x27);
		assert_eq!(cpu.sp, 0xff);
	}

	#[test]
	fn state_after_nmi () {
		let mut cpu = Mos6502::new();
		let mut mem: Ram<u16> = Ram::new();
		cpu.step(&mut mem);
		assert!(!cpu.reset && !cpu.nmi && !cpu.irq);
		cpu.sr = 0x23;
		cpu.sp = 0xff;
		mem.set_le(0xfffa, 0x1234);
		cpu.nmi();
		cpu.step(&mut mem);
		assert_eq!(cpu.pc, 0x1234);
		assert_eq!(cpu.sr, 0x23);
		assert_eq!(cpu.sp, 0xfc);
	}

	#[test]
	fn state_after_irq () {
		let mut cpu = Mos6502::new();
		let mut mem: Ram<u16> = Ram::new();
		cpu.step(&mut mem);
		assert!(!cpu.reset && !cpu.nmi && !cpu.irq);
		cpu.sr = 0x23;
		cpu.sp = 0xff;
		mem.set_le(0xfffe, 0x1234);
		cpu.irq();
		cpu.step(&mut mem);
		assert_eq!(cpu.pc, 0x1234);
		assert_eq!(cpu.sr, 0x27);
		assert_eq!(cpu.sp, 0xfc);
	}
}
