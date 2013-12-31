use mem::Addressable;
use super::cpu::CPU;

// General information on 65xx: http://en.wikipedia.org/wiki/MOS_Technology_6510
// Useful emulator information: http://emudocs.org/?page=CPU%2065xx
// Web simulator and much info: http://e-tradition.net/bytes/6502/
// Good reference and overview: http://www.obelisk.demon.co.uk/index.html
// Processor bugs and caveats : http://www.textfiles.com/apple/6502.bugs.txt

/// Processor instructions
#[deriving(Eq)]
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
#[deriving(Eq)]
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
			Implied								=> ~"",
			Immediate(value)					=> format!("\\#${:02X}", value),
			Accumulator							=> ~"A",
			Relative(offset)					=> format!("{:+d}", offset),
			Absolute(addr)						=> format!("${:04X}", addr),
			AbsoluteIndexedWithX(addr)			=> format!("${:04X},X", addr),
			AbsoluteIndexedWithY(addr)			=> format!("${:04X},Y", addr),
			Indirect(addr)						=> format!("(${:04X})", addr),
			ZeroPage(addr)						=> format!("${:02X}", addr),
			ZeroPageIndexedWithX(addr)			=> format!("${:02X},X", addr),
			ZeroPageIndexedWithY(addr)			=> format!("${:02X},Y", addr),
			ZeroPageIndexedWithXIndirect(addr)	=> format!("(${:02X},X)", addr),
			ZeroPageIndirectIndexedWithY(addr)	=> format!("(${:02X}),Y", addr),
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

	/// Parse next instruction and advance PC. Returns number of cycles, instruction and operand
	fn next_instruction<M: Addressable<u16>> (&mut self, mem: &M) -> Option<(uint, Instruction, Operand)> {
		let opcode: u8 = self.next(mem);
		Some(match opcode {
			// Load/store operations
			0xa9 => (2, LDA, Immediate(self.next(mem))),
			0xa5 => (3, LDA, ZeroPage(self.next(mem))),
			0xb5 => (4, LDA, ZeroPageIndexedWithX(self.next(mem))),
			0xad => (4, LDA, Absolute(self.next(mem))),
			0xbd => (4, LDA, AbsoluteIndexedWithX(self.next(mem))),				// +1 cycle if page crossed
			0xb9 => (4, LDA, AbsoluteIndexedWithY(self.next(mem))),				// +1 cycle if page crossed
			0xa1 => (6, LDA, ZeroPageIndexedWithXIndirect(self.next(mem))),
			0xb1 => (5, LDA, ZeroPageIndirectIndexedWithY(self.next(mem))),		// +1 cycle if page crossed
			0xa2 => (2, LDX, Immediate(self.next(mem))),
			0xa6 => (3, LDX, ZeroPage(self.next(mem))),
			0xb6 => (4, LDX, ZeroPageIndexedWithY(self.next(mem))),
			0xae => (4, LDX, Absolute(self.next(mem))),
			0xbe => (4, LDX, AbsoluteIndexedWithY(self.next(mem))),				// +1 cycle if page crossed
			0xa0 => (2, LDY, Immediate(self.next(mem))),
			0xa4 => (3, LDY, ZeroPage(self.next(mem))),
			0xb4 => (4, LDY, ZeroPageIndexedWithX(self.next(mem))),
			0xac => (4, LDY, Absolute(self.next(mem))),
			0xbc => (4, LDY, AbsoluteIndexedWithX(self.next(mem))),				// +1 cycle if page crossed
			0x85 => (3, STA, ZeroPage(self.next(mem))),
			0x95 => (4, STA, ZeroPageIndexedWithX(self.next(mem))),
			0x8d => (4, STA, Absolute(self.next(mem))),
			0x9d => (5, STA, AbsoluteIndexedWithX(self.next(mem))),
			0x99 => (5, STA, AbsoluteIndexedWithY(self.next(mem))),
			0x81 => (6, STA, ZeroPageIndexedWithXIndirect(self.next(mem))),
			0x91 => (6, STA, ZeroPageIndirectIndexedWithY(self.next(mem))),
			0x86 => (3, STX, ZeroPage(self.next(mem))),
			0x96 => (4, STX, ZeroPageIndexedWithY(self.next(mem))),
			0x8e => (4, STX, Absolute(self.next(mem))),
			0x84 => (3, STY, ZeroPage(self.next(mem))),
			0x94 => (4, STY, ZeroPageIndexedWithX(self.next(mem))),
			0x8c => (4, STY, Absolute(self.next(mem))),
			// Register transfers
			0xaa => (2, TAX, Implied),
			0xa8 => (2, TAY, Implied),
			0x8a => (2, TXA, Implied),
			0x98 => (2, TYA, Implied),
			// Stack operations
			0xba => (2, TSX, Implied),
			0x9a => (2, TXS, Implied),
			0x48 => (3, PHA, Implied),
			0x08 => (3, PHP, Implied),
			0x68 => (4, PLA, Implied),
			0x28 => (4, PLP, Implied),
			// Logical
			0x29 => (2, AND, Immediate(self.next(mem))),
			0x25 => (3, AND, ZeroPage(self.next(mem))),
			0x35 => (4, AND, ZeroPageIndexedWithX(self.next(mem))),
			0x2d => (4, AND, Absolute(self.next(mem))),
			0x3d => (4, AND, AbsoluteIndexedWithX(self.next(mem))),				// +1 cycle if page crossed
			0x39 => (4, AND, AbsoluteIndexedWithY(self.next(mem))),				// +1 cycle if page crossed
			0x21 => (6, AND, ZeroPageIndexedWithXIndirect(self.next(mem))),
			0x31 => (5, AND, ZeroPageIndirectIndexedWithY(self.next(mem))),		// +1 cycle if page crossed
			0x49 => (2, EOR, Immediate(self.next(mem))),
			0x45 => (3, EOR, ZeroPage(self.next(mem))),
			0x55 => (4, EOR, ZeroPageIndexedWithX(self.next(mem))),
			0x4d => (4, EOR, Absolute(self.next(mem))),
			0x5d => (4, EOR, AbsoluteIndexedWithX(self.next(mem))),				// +1 cycle if page crossed
			0x59 => (4, EOR, AbsoluteIndexedWithY(self.next(mem))),				// +1 cycle if page crossed
			0x41 => (6, EOR, ZeroPageIndexedWithXIndirect(self.next(mem))),
			0x51 => (5, EOR, ZeroPageIndirectIndexedWithY(self.next(mem))),		// +1 cycle if page crossed
			0x09 => (2, ORA, Immediate(self.next(mem))),
			0x05 => (3, ORA, ZeroPage(self.next(mem))),
			0x15 => (4, ORA, ZeroPageIndexedWithX(self.next(mem))),
			0x0d => (4, ORA, Absolute(self.next(mem))),
			0x1d => (4, ORA, AbsoluteIndexedWithX(self.next(mem))),				// +1 cycle if page crossed
			0x19 => (4, ORA, AbsoluteIndexedWithY(self.next(mem))),				// +1 cycle if page crossed
			0x01 => (6, ORA, ZeroPageIndexedWithXIndirect(self.next(mem))),
			0x11 => (5, ORA, ZeroPageIndirectIndexedWithY(self.next(mem))),		// +1 cycle if page crossed
			0x24 => (3, BIT, ZeroPage(self.next(mem))),
			0x2c => (4, BIT, Absolute(self.next(mem))),
			// Arithmetic
			0x69 => (2, ADC, Immediate(self.next(mem))),
			0x65 => (3, ADC, ZeroPage(self.next(mem))),
			0x75 => (4, ADC, ZeroPageIndexedWithX(self.next(mem))),
			0x6d => (4, ADC, Absolute(self.next(mem))),
			0x7d => (4, ADC, AbsoluteIndexedWithX(self.next(mem))),				// +1 cycle if page crossed
			0x79 => (4, ADC, AbsoluteIndexedWithY(self.next(mem))),				// +1 cycle if page crossed
			0x61 => (6, ADC, ZeroPageIndexedWithXIndirect(self.next(mem))),
			0x71 => (5, ADC, ZeroPageIndirectIndexedWithY(self.next(mem))),		// +1 cycle if page crossed
			0xe9 => (2, SBC, Immediate(self.next(mem))),
			0xe5 => (3, SBC, ZeroPage(self.next(mem))),
			0xf5 => (4, SBC, ZeroPageIndexedWithX(self.next(mem))),
			0xed => (4, SBC, Absolute(self.next(mem))),
			0xfd => (4, SBC, AbsoluteIndexedWithX(self.next(mem))),				// +1 cycle if page crossed
			0xf9 => (4, SBC, AbsoluteIndexedWithY(self.next(mem))),				// +1 cycle if page crossed
			0xe1 => (6, SBC, ZeroPageIndexedWithXIndirect(self.next(mem))),
			0xf1 => (5, SBC, ZeroPageIndirectIndexedWithY(self.next(mem))),		// +1 cycle if page crossed
			0xc9 => (2, CMP, Immediate(self.next(mem))),
			0xc5 => (3, CMP, ZeroPage(self.next(mem))),
			0xd5 => (4, CMP, ZeroPageIndexedWithX(self.next(mem))),
			0xcd => (4, CMP, Absolute(self.next(mem))),
			0xdd => (4, CMP, AbsoluteIndexedWithX(self.next(mem))),				// +1 cycle if page crossed
			0xd9 => (4, CMP, AbsoluteIndexedWithY(self.next(mem))),				// +1 cycle if page crossed
			0xc1 => (6, CMP, ZeroPageIndexedWithXIndirect(self.next(mem))),
			0xd1 => (5, CMP, ZeroPageIndirectIndexedWithY(self.next(mem))),		// +1 cycle if page crossed
			0xe0 => (2, CPX, Immediate(self.next(mem))),
			0xe4 => (3, CPX, ZeroPage(self.next(mem))),
			0xec => (4, CPX, Absolute(self.next(mem))),
			0xc0 => (2, CPY, Immediate(self.next(mem))),
			0xc4 => (3, CPY, ZeroPage(self.next(mem))),
			0xcc => (4, CPY, Absolute(self.next(mem))),
			// Increments & decrements
			0xe6 => (5, INC, ZeroPage(self.next(mem))),
			0xf6 => (6, INC, ZeroPageIndexedWithX(self.next(mem))),
			0xee => (6, INC, Absolute(self.next(mem))),
			0xfe => (7, INC, AbsoluteIndexedWithX(self.next(mem))),
			0xe8 => (2, INX, Implied),
			0xc8 => (2, INY, Implied),
			0xc6 => (5, DEC, ZeroPage(self.next(mem))),
			0xd6 => (6, DEC, ZeroPageIndexedWithX(self.next(mem))),
			0xce => (6, DEC, Absolute(self.next(mem))),
			0xde => (7, DEC, AbsoluteIndexedWithX(self.next(mem))),
			0xca => (2, DEX, Implied),
			0x88 => (2, DEY, Implied),
			// Shifts
			0x0a => (2, ASL, Accumulator),
			0x06 => (5, ASL, ZeroPage(self.next(mem))),
			0x16 => (6, ASL, ZeroPageIndexedWithX(self.next(mem))),
			0x0e => (6, ASL, Absolute(self.next(mem))),
			0x1e => (7, ASL, AbsoluteIndexedWithX(self.next(mem))),
			0x4a => (2, LSR, Accumulator),
			0x46 => (5, LSR, ZeroPage(self.next(mem))),
			0x56 => (6, LSR, ZeroPageIndexedWithX(self.next(mem))),
			0x4e => (6, LSR, Absolute(self.next(mem))),
			0x5e => (7, LSR, AbsoluteIndexedWithX(self.next(mem))),
			0x2a => (2, ROL, Accumulator),
			0x26 => (5, ROL, ZeroPage(self.next(mem))),
			0x36 => (6, ROL, ZeroPageIndexedWithX(self.next(mem))),
			0x2e => (6, ROL, Absolute(self.next(mem))),
			0x3e => (7, ROL, AbsoluteIndexedWithX(self.next(mem))),
			0x6a => (2, ROR, Accumulator),
			0x66 => (5, ROR, ZeroPage(self.next(mem))),
			0x76 => (6, ROR, ZeroPageIndexedWithX(self.next(mem))),
			0x6e => (6, ROR, Absolute(self.next(mem))),
			0x7e => (7, ROR, AbsoluteIndexedWithX(self.next(mem))),
			// Jump & calls
			0x4c => (3, JMP, Absolute(self.next(mem))),
			0x6c => (5, JMP, Indirect(self.next(mem))),
			0x20 => (6, JSR, Absolute(self.next(mem))),
			0x60 => (6, RTS, Implied),
			// Branches
			0x90 => (2, BCC, Relative(self.next(mem))),							// +1 cycle if branched, +2 if page crossed
			0xb0 => (2, BCS, Relative(self.next(mem))),							// +1 cycle if branched, +2 if page crossed
			0xf0 => (2, BEQ, Relative(self.next(mem))),							// +1 cycle if branched, +2 if page crossed
			0x30 => (2, BMI, Relative(self.next(mem))),							// +1 cycle if branched, +2 if page crossed
			0xd0 => (2, BNE, Relative(self.next(mem))),							// +1 cycle if branched, +2 if page crossed
			0x10 => (2, BPL, Relative(self.next(mem))),							// +1 cycle if branched, +2 if page crossed
			0x50 => (2, BVC, Relative(self.next(mem))),							// +1 cycle if branched, +2 if page crossed
			0x70 => (2, BVS, Relative(self.next(mem))),							// +1 cycle if branched, +2 if page crossed
			// Status flag changes
			0x18 => (2, CLC, Implied),
			0xd8 => (2, CLD, Implied),
			0x58 => (2, CLI, Implied),
			0xb8 => (2, CLV, Implied),
			0x38 => (2, SEC, Implied),
			0xf8 => (2, SED, Implied),
			0x78 => (2, SEI, Implied),
			// System functions
			0x00 => (7, BRK, Implied),
			0xea => (2, NOP, Implied),
			0x40 => (6, RTI, Implied),
			// Illegal opcode
			_ => return None,
		})
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
		// Helper function to print a hexdump with up to 4 bytes
		fn hexdump<M: Addressable<u16>> (mem: &M, addr: u16, addr2: u16) -> ~str {
			match addr2 - addr {
				0 => ~"",
				1 => format!("{:02X}", mem.get(addr)),
				2 => format!("{:02X} {:02X}", mem.get(addr), mem.get(addr+1)),
				3 => format!("{:02X} {:02X} {:02X}", mem.get(addr), mem.get(addr+1), mem.get(addr+2)),
				4 => format!("{:02X} {:02X} {:02X} {:02X}", mem.get(addr), mem.get(addr+1), mem.get(addr+2), mem.get(addr+3)),
				_ => unreachable!(),
			}
		}
		// Read and parse next opcode
		let old_pc = self.pc;
		match self.next_instruction(mem) {
			// Got valid opcode
			Some((cycles, instruction, operand)) => {
				let new_pc = self.pc;

				// TODO: Execute the instruction
				//instruction.execute(self, mem, operand)

				debug!("mos6502: {:04X}  {:-8s}  {:-3s} {:-26s}  -[{:u}]-> AC:{:02X} X:{:02X} Y:{:02X} SR:{:02X} SP:{:02X} NV-BDIZC:{:08t}",
					old_pc, hexdump(mem, old_pc, new_pc), instruction.as_str(), operand.as_str(),
					cycles, self.ac, self.x, self.y, self.sr, self.sp, self.sr);
				cycles
			},
			// Got illegal opcode
			None => {
				debug!("mos6502: {:04X}  {:-8s}  ???", old_pc, hexdump(mem, old_pc, old_pc+3));
				fail!("mos6502: Illegal opcode ${:02X} at ${:04X}", mem.get(old_pc), old_pc)
			},
		}
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
		cpu.pc = 0x0012;
		let val: u8 = cpu.next(&mem); assert_eq!(val, 0x12);
		let val: u8 = cpu.next(&mem); assert_eq!(val, 0x13);
		let val: u16 = cpu.next(&mem); assert_eq!(val, 0x1514);
		let val: u16 = cpu.next(&mem); assert_eq!(val, 0x1716);
	}

	#[test]
	fn fetch_instruction_and_advance_pc () {
		let mut cpu = Mos6502::new();
		let mut mem = TestMemory;
		cpu.pc = 0x00ad;					// AD AE AF: LDA $AFAE
		assert_eq!(cpu.next_instruction(&mut mem), Some((4, LDA, Absolute(0xafae))));
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
