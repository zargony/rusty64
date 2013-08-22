use addressable::Addressable;
use std::sys;

#[cfg(test)]
use testmemory::TestMemory;


enum Operand {
	Implied,							// OPC				// operand implied
	Immediate(u8),						// OPC #$BB			// operand is byte (BB)
	Accumulator,						// OPC A			// operand is AC
	Relative(i8),						// OPC $BB			// branch target is PC + offset (BB), bit 7 signifies negative offset
	Absolute(u16),						// OPC $HHLL		// operand is address $HHLL
	AbsoluteIndexedWithX(u16),			// OPC $HHLL,X		// operand is address $HHLL incremented by X with carry
	AbsoluteIndexedWithY(u16),			// OPC $HHLL,Y		// operand is address $HHLL incremented by Y with carry
	Indirect(u16),						// OPC ($HHLL)		// operand is effective address; effective address is value of address
	ZeroPage(u8),						// OPC $LL			// operand is address $00LL
	ZeroPageIndexedWithX(u8),			// OPC $LL,X		// operand is address $00LL incremented by X, no page transition
	ZeroPageIndexedWithY(u8),			// OPC $LL,Y		// operand is address $00LL incremented by Y, no page transition
	ZeroPageIndexedWithXIndirect(u8),	// OPC ($LL,X)		// operand is effective address; effective address is $00LL incremented by X without carry
	ZeroPageIndirectIndexedWithY(u8),	// OPC ($LL),Y		// operand is effective address incremented by Y with carry; effective address is word at $00LL
}

impl Operand {
	fn indirect<M: Addressable<u16>> (mem: &M, addr: u16) -> u16 {
		// Get a little endian u16 from the given address, simulating the 6502 MSB bug
		// (i.e. take values from $c0ff/$c000 instead of $c0ff/$c100)
		let addr1 = (addr & 0xff00) | (addr+1 & 0x00ff);
		mem.get(addr) as u16 | mem.get(addr1) as u16 << 8
	}

	fn addr<M: Addressable<u16>> (&self, cpu: &Mos6502, mem: &M) -> u16 {
		match *self {
			Implied								=> fail!("mos65xx: Implied operand is never targetted to an address"),
			Immediate(_)						=> fail!("mos65xx: Immediate operand is never targetted to an address"),
			Accumulator							=> fail!("mos65xx: Accumulator operand is never targetted to an address"),
			Relative(offset)					=> cpu.pc + offset as u16,
			Absolute(addr)						=> addr,
			AbsoluteIndexedWithX(addr)			=> addr + cpu.x as u16,
			AbsoluteIndexedWithY(addr)			=> addr + cpu.y as u16,
			Indirect(addr)						=> Operand::indirect(mem, addr),
			ZeroPage(addr)						=> addr as u16,
			ZeroPageIndexedWithX(addr)			=> (addr + cpu.x) as u16,						// no page transition
			ZeroPageIndexedWithY(addr)			=> (addr + cpu.y) as u16,						// no page transition
			ZeroPageIndexedWithXIndirect(addr)	=> mem.get_le((addr + cpu.x) as u16),			// no page transition
			ZeroPageIndirectIndexedWithY(addr)	=> { let iaddr: u16 = mem.get_le(addr as u16); iaddr + cpu.y as u16 },
		}
	}

	fn get<M: Addressable<u16>> (&self, cpu: &Mos6502, mem: &M) -> u8 {
		match *self {
			Implied								=> fail!("mos65xx: Implied operand never has a value"),
			Immediate(value)					=> value,
			Accumulator							=> cpu.ac,
			Relative(_target)					=> fail!("mos65xx: Relative operand never has a value"),
			op									=> { let addr = op.addr(cpu, mem); mem.get(addr) },
		}
	}

	fn set<M: Addressable<u16>> (&self, cpu: &mut Mos6502, mem: &mut M, value: u8) {
		match *self {
			Implied								=> fail!("mos65xx: Implied operand never sets a value"),
			Immediate(_)						=> fail!("mos65xx: Immediate operand never sets a value"),
			Accumulator							=> cpu.ac = value,
			Relative(_target)					=> fail!("mos65xx: Relative operand never sets a value"),
			op									=> { let addr = op.addr(cpu, mem); mem.set(addr, value); },
		}
	}

	fn as_str (&self) -> ~str {
		match *self {
			Implied => ~"",
			Immediate(value) => fmt!("#$%02X", value as uint),
			Accumulator => ~"A",
			Relative(offset) => fmt!("%c$%02X", if offset < 0 { '-' } else { '+' }, offset.abs() as uint),
			Absolute(addr) => fmt!("$%04X", addr as uint),
			AbsoluteIndexedWithX(addr) => fmt!("$%04X,X", addr as uint),
			AbsoluteIndexedWithY(addr) => fmt!("$%04X,Y", addr as uint),
			Indirect(addr) => fmt!("($%04X)", addr as uint),
			ZeroPage(addr) => fmt!("$%02X", addr as uint),
			ZeroPageIndexedWithX(addr) => fmt!("$%02X,X", addr as uint),
			ZeroPageIndexedWithY(addr) => fmt!("$%02X,Y", addr as uint),
			ZeroPageIndexedWithXIndirect(addr) => fmt!("($%02X,X)", addr as uint),
			ZeroPageIndirectIndexedWithY(addr) => fmt!("($%02X),Y", addr as uint),
		}
	}
}


enum Instruction {
	// Load/store operations
	LDA,								// load accumulator [N,Z]
	LDX,								// load X register [N,Z]
	LDY,								// load Y register [N,Z]
	STA,								// store accumulator
	STX,								// store X register
	STY,								// store Y register
	// Register transfers
	TAX,								// transfer accumulator to X [N,Z]
	TAY,								// transfer accumulator to Y [N,Z]
	TXA,								// transfer X to accumulator [N,Z]
	TYA,								// transfer Y to accumulator [N,Z]
	// Stack operations
	TSX,								// transfer stack pointer to X [N,Z]
	TXS,								// transfer X to stack pointer
	PHA,								// push accumulator on stack
	PHP,								// push processor status (SR) on stack
	PLA,								// pull accumulator from stack [N,Z]
	PLP,								// pull processor status (SR) from stack [all]
	// Logical
	AND,								// logical AND [N,Z]
	EOR,								// exclusive OR [N,Z]
	ORA,								// logical inclusive OR [N,Z]
	BIT,								// bit test [N,V,Z]
	// Arithmetic
	ADC,								// add with carry [N,V,Z,C]
	SBC,								// subtract with carry [N,V,Z,C]
	CMP,								// compare (with accumulator) [N,Z,C]
	CPX,								// compare with X register [N,Z,C]
	CPY,								// compare with Y register [N,Z,C]
	// Increments & decrements
	INC,								// increment a memory location [N,Z]
	INX,								// increment X register [N,Z]
	INY,								// increment Y register [N,Z]
	DEC,								// decrement a memory location [N,Z]
	DEX,								// decrement X register [N,Z]
	DEY,								// decrement Y register [N,Z]
	// Shifts
	ASL,								// arithmetic shift left [N,Z,C]
	LSR,								// logical shift right [N,Z,C]
	ROL,								// rotate left [N,Z,C]
	ROR,								// rotate right [N,Z,C]
	// Jump & calls
	JMP,								// jump to another location
	JSR,								// jump to a subroutine
	RTS,								// return from subroutine
	// Branches
	BCC,								// branch if carry flag clear
	BCS,								// branch if carry flag set
	BEQ,								// branch if zero flag set
	BMI,								// branch if negative flag set
	BNE,								// branch if zero flag clear
	BPL,								// branch if negative flag clear
	BVC,								// branch if overflow flag clear
	BVS,								// branch if overflow flag set
	// Status flag changes
	CLC,								// clear carry flag [C]
	CLD,								// clear decimal mode flag [D]
	CLI,								// clear interrupt disable flag [I]
	CLV,								// clear overflow flag [V]
	SEC,								// set carry flag [C]
	SED,								// set decimal mode flag [D]
	SEI,								// set interrupt disable flag [I]
	// System functions
	BRK,								// force an interrupt [B]
	NOP,								// no operation
	RTI,								// return from interrupt [all]
}

impl Instruction {
	fn execute<M: Addressable<u16>> (&self, operand: &Operand, cpu: &mut Mos6502, mem: &mut M) {
		match *self {
			// Load/store operations
			LDA => {
				cpu.ac = operand.get(cpu, mem);
				cpu.set_zn(cpu.ac);
			},
			LDX => {
				cpu.x = operand.get(cpu, mem);
				cpu.set_zn(cpu.x);
			},
			LDY => {
				cpu.y = operand.get(cpu, mem);
				cpu.set_zn(cpu.y);
			},
			STA => {
				operand.set(cpu, mem, cpu.ac);
			},
			STX => {
				operand.set(cpu, mem, cpu.x);
			},
			STY => {
				operand.set(cpu, mem, cpu.y);
			},
			// Register transfers
			TAX => {
				cpu.x = cpu.ac;
				cpu.set_zn(cpu.x);
			},
			TAY => {
				cpu.y = cpu.ac;
				cpu.set_zn(cpu.y);
			},
			TXA => {
				cpu.ac = cpu.x;
				cpu.set_zn(cpu.ac);
			},
			TYA => {
				cpu.ac = cpu.y;
				cpu.set_zn(cpu.ac);
			},
			// Stack operations
			TSX => {
				cpu.x = cpu.sp;
				cpu.set_zn(cpu.x);
			},
			TXS => {
				cpu.sp = cpu.x;
			},
			PHA => {
				fail!("mos65xx: PHA instruction not implemented yet");				// TODO
			},
			PHP => {
				fail!("mos65xx: PHP instruction not implemented yet");				// TODO
			},
			PLA => {
				fail!("mos65xx: PLA instruction not implemented yet");				// TODO
			},
			PLP => {
				fail!("mos65xx: PLP instruction not implemented yet");				// TODO
			},
			// Logical
			AND => {
				cpu.ac &= operand.get(cpu, mem);
				cpu.set_zn(cpu.ac);
			},
			EOR => {
				cpu.ac ^= operand.get(cpu, mem);
				cpu.set_zn(cpu.ac);
			},
			ORA => {
				cpu.ac |= operand.get(cpu, mem);
				cpu.set_zn(cpu.ac);
			},
			BIT => {
				let value = operand.get(cpu, mem);
				cpu.set_flag(ZeroFlag, (value & cpu.ac) == 0);
				cpu.set_flag(NegativeFlag, (value & 0x80) != 0);
				cpu.set_flag(OverflowFlag, (value & 0x40) != 0);
			},
			// Arithmetic
			ADC => {
				fail!("mos65xx: ADC instruction not implemented yet");				// TODO
			},
			SBC => {
				fail!("mos65xx: SBC instruction not implemented yet");				// TODO
			},
			CMP => {
				let result = cpu.ac as i16 - operand.get(cpu, mem) as i16;
				cpu.set_flag(CarryFlag, result >= 0);
				cpu.set_zn(result as u8);
			},
			CPX => {
				let result = cpu.x as i16 - operand.get(cpu, mem) as i16;
				cpu.set_flag(CarryFlag, result >= 0);
				cpu.set_zn(result as u8);
			},
			CPY => {
				let result = cpu.y as i16 - operand.get(cpu, mem) as i16;
				cpu.set_flag(CarryFlag, result >= 0);
				cpu.set_zn(result as u8);
			},
			// Increments & decrements
			INC => {
				let value = operand.get(cpu, mem) + 1;
				operand.set(cpu, mem, value);
				cpu.set_zn(value);
			},
			INX => {
				cpu.x += 1;
				cpu.set_zn(cpu.x);
			},
			INY => {
				cpu.y += 1;
				cpu.set_zn(cpu.y);
			},
			DEC => {
				let value = operand.get(cpu, mem) - 1;
				operand.set(cpu, mem, value);
				cpu.set_zn(value);
			},
			DEX => {
				cpu.x -= 1;
				cpu.set_zn(cpu.x);
			},
			DEY => {
				cpu.y -= 1;
				cpu.set_zn(cpu.y);
			},
			// Shifts
			ASL => {
				let value = operand.get(cpu, mem);
				cpu.set_flag(CarryFlag, (value & 0x80) != 0);
				let result = value << 1;
				operand.set(cpu, mem, result);
				cpu.set_zn(result);
			},
			LSR => {
				let value = operand.get(cpu, mem);
				cpu.set_flag(CarryFlag, (value & 0x01) != 0);
				let result = value >> 1;
				operand.set(cpu, mem, result);
				cpu.set_zn(result);
			},
			ROL => {
				let carry = cpu.get_flag(CarryFlag);
				let value = operand.get(cpu, mem);
				cpu.set_flag(CarryFlag, (value & 0x80) != 0);
				let mut result = value << 1;
				if carry { result |= 0x01 }
				operand.set(cpu, mem, result);
				cpu.set_zn(result);
			},
			ROR => {
				let carry = cpu.get_flag(CarryFlag);
				let value = operand.get(cpu, mem);
				cpu.set_flag(CarryFlag, (value & 0x01) != 0);
				let mut result = value >> 1;
				if carry { result |= 0x80 }
				operand.set(cpu, mem, result);
				cpu.set_zn(result);
			},
			// Jump & calls
			JMP => {
				cpu.pc = operand.addr(cpu, mem);
			},
			JSR => {
				cpu.push(mem, cpu.pc - 1);
				cpu.pc = operand.addr(cpu, mem);
			},
			RTS => {
				cpu.pc = cpu.pop(mem);
				cpu.pc += 1;
			},
			// Branches
			BCC => {
				if !cpu.get_flag(CarryFlag) {
					cpu.pc = operand.addr(cpu, mem);
				}
			},
			BCS => {
				if cpu.get_flag(CarryFlag) {
					cpu.pc = operand.addr(cpu, mem);
				}
			},
			BEQ => {
				if cpu.get_flag(ZeroFlag) {
					cpu.pc = operand.addr(cpu, mem);
				}
			},
			BMI => {
				if cpu.get_flag(NegativeFlag) {
					cpu.pc = operand.addr(cpu, mem);
				}
			},
			BNE => {
				if !cpu.get_flag(ZeroFlag) {
					cpu.pc = operand.addr(cpu, mem);
				}
			},
			BPL => {
				if !cpu.get_flag(NegativeFlag) {
					cpu.pc = operand.addr(cpu, mem);
				}
			},
			BVC => {
				if !cpu.get_flag(OverflowFlag) {
					cpu.pc = operand.addr(cpu, mem);
				}
			},
			BVS => {
				if cpu.get_flag(OverflowFlag) {
					cpu.pc = operand.addr(cpu, mem);
				}
			},
			// Status flag changes
			CLC => {
				cpu.set_flag(CarryFlag, false);
			},
			CLD => {
				cpu.set_flag(DecimalFlag, false);
			},
			CLI => {
				cpu.set_flag(InterruptDisableFlag, false);
			},
			CLV => {
				cpu.set_flag(OverflowFlag, false);
			},
			SEC => {
				cpu.set_flag(CarryFlag, true);
			},
			SED => {
				cpu.set_flag(DecimalFlag, true);
			},
			SEI => {
				cpu.set_flag(InterruptDisableFlag, true);
			},
			// System functions
			BRK => {
				fail!("mos65xx: BRK instruction not implemented yet");				// TODO
			},
			NOP => {
			},
			RTI => {
				fail!("mos65xx: RTI instruction not implemented yet");				// TODO
			},
		}
	}

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


// General information on 65xx: http://en.wikipedia.org/wiki/MOS_Technology_6510
// Web simulator and much info: http://e-tradition.net/bytes/6502/
// Good reference and overview: http://www.obelisk.demon.co.uk/index.html
// Processor bugs and caveats : http://www.textfiles.com/apple/6502.bugs.txt

enum StatusFlag {
	CarryFlag = 0,
	ZeroFlag = 1,
	InterruptDisableFlag = 2,
	DecimalFlag = 3,
	BreakFlag = 4,
	OverflowFlag = 6,
	NegativeFlag = 7,
}

static NMI_VECTOR:       u16 = 0xfffa;
static RESET_VECTOR:     u16 = 0xfffc;
static IRQ_VECTOR:       u16 = 0xfffe;

pub struct Mos6502 {
	priv pc: u16,						// program counter
	priv ac: u8,						// accumulator
	priv x: u8,							// x register
	priv y: u8,							// y register
	priv sr: u8,						// status register (NV-BDIZC: Negative, oVerflow, 1, Break, Decimal, Interrupt, Zero, Carry)
	priv sp: u8,						// stack pointer
}

impl Mos6502 {
	pub fn new () -> Mos6502 {
		Mos6502 { pc: 0, ac: 0, x: 0, y: 0, sr: 0, sp: 0 }
	}

	fn get_flag (&self, flag: StatusFlag) -> bool {
		(self.sr & (1 << flag as u8)) != 0
	}

	fn set_flag (&mut self, flag: StatusFlag, on: bool) {
		if on {
			self.sr |= (1 << flag as u8);
		} else {
			self.sr &= !(1 << flag as u8);
		}
	}

	fn set_zn (&mut self, value: u8) -> u8 {
		self.set_flag(ZeroFlag, value == 0);
		self.set_flag(NegativeFlag, (value as i8) < 0);
		value
	}

	fn push<M: Addressable<u16>, T: Int> (&mut self, mem: &mut M, value: T) {
		self.sp -= sys::size_of::<T>() as u8;
		mem.set_le(0x0100 + self.sp as u16 + 1, value);
	}

	fn pop<M: Addressable<u16>, T: Int> (&mut self, mem: &M) -> T {
		let value: T = mem.get_le(0x0100 + self.sp as u16 + 1);
		self.sp += sys::size_of::<T>() as u8;
		value
	}

	fn get_opcode<M: Addressable<u16>> (&mut self, mem: &M) -> u8 {
		let opcode = mem.get(self.pc);
		self.pc += 1;
		opcode
	}

	fn get_argument<M: Addressable<u16>, T: Int> (&mut self, mem: &M) -> T {
		let argument: T = mem.get_le(self.pc);
		self.pc += sys::size_of::<T>() as u16;
		argument
	}

	pub fn reset<M: Addressable<u16>> (&mut self, mem: &M) {
		// On reset, the interrupt-disable flag is set (and the decimal flag is cleared in the CMOS version 65c02).
		// The other bits and all registers (including the stack pointer are unspecified and might contain random values.
		// Execution begins at the address pointed to by the reset vector at address $FFFC.
		self.pc = mem.get_le(RESET_VECTOR);
		self.sr = 0x24;
		debug!("mos65xx: Reset! Start at ($%04X) -> $%04X", RESET_VECTOR as uint, self.pc as uint);
	}

	pub fn step<M: Addressable<u16>> (&mut self, mem: &mut M) -> uint {
		let old_pc = self.pc;
		let opcode = self.get_opcode(mem);
		let (instruction, operand, cycles) = match opcode {
			// Load/store operations
			0xa9 => (LDA, Immediate(self.get_argument(mem)), 2),
			0xa5 => (LDA, ZeroPage(self.get_argument(mem)), 3),
			0xb5 => (LDA, ZeroPageIndexedWithX(self.get_argument(mem)), 4),
			0xad => (LDA, Absolute(self.get_argument(mem)), 4),
			0xbd => (LDA, AbsoluteIndexedWithX(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0xb9 => (LDA, AbsoluteIndexedWithY(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0xa1 => (LDA, ZeroPageIndexedWithXIndirect(self.get_argument(mem)), 6),
			0xb1 => (LDA, ZeroPageIndirectIndexedWithY(self.get_argument(mem)), 5),		// FIXME: +1 cycle if page crossed
			0xa2 => (LDX, Immediate(self.get_argument(mem)), 2),
			0xa6 => (LDX, ZeroPage(self.get_argument(mem)), 3),
			0xb6 => (LDX, ZeroPageIndexedWithY(self.get_argument(mem)), 4),
			0xae => (LDX, Absolute(self.get_argument(mem)), 4),
			0xbe => (LDX, AbsoluteIndexedWithY(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0xa0 => (LDY, Immediate(self.get_argument(mem)), 2),
			0xa4 => (LDY, ZeroPage(self.get_argument(mem)), 3),
			0xb4 => (LDY, ZeroPageIndexedWithX(self.get_argument(mem)), 4),
			0xac => (LDY, Absolute(self.get_argument(mem)), 4),
			0xbc => (LDY, AbsoluteIndexedWithX(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0x85 => (STA, ZeroPage(self.get_argument(mem)), 3),
			0x95 => (STA, ZeroPageIndexedWithX(self.get_argument(mem)), 4),
			0x8d => (STA, Absolute(self.get_argument(mem)), 4),
			0x9d => (STA, AbsoluteIndexedWithX(self.get_argument(mem)), 5),
			0x99 => (STA, AbsoluteIndexedWithY(self.get_argument(mem)), 5),
			0x81 => (STA, ZeroPageIndexedWithXIndirect(self.get_argument(mem)), 6),
			0x91 => (STA, ZeroPageIndirectIndexedWithY(self.get_argument(mem)), 6),
			0x86 => (STX, ZeroPage(self.get_argument(mem)), 3),
			0x96 => (STX, ZeroPageIndexedWithY(self.get_argument(mem)), 4),
			0x8e => (STX, Absolute(self.get_argument(mem)), 4),
			0x84 => (STY, ZeroPage(self.get_argument(mem)), 3),
			0x94 => (STY, ZeroPageIndexedWithX(self.get_argument(mem)), 4),
			0x8c => (STY, Absolute(self.get_argument(mem)), 4),
			// Register transfers
			0xaa => (TAX, Implied, 2),
			0xa8 => (TAY, Implied, 2),
			0x8a => (TXA, Implied, 2),
			0x98 => (TYA, Implied, 2),
			0xba => (TSX, Implied, 2),
			0x9a => (TXS, Implied, 2),
			// Stack operations
			0x48 => (PHA, Implied, 3),
			0x08 => (PHP, Implied, 3),
			0x68 => (PLA, Implied, 4),
			0x28 => (PLP, Implied, 4),
			// Logical
			0x29 => (AND, Immediate(self.get_argument(mem)), 2),
			0x25 => (AND, ZeroPage(self.get_argument(mem)), 3),
			0x35 => (AND, ZeroPageIndexedWithX(self.get_argument(mem)), 4),
			0x2d => (AND, Absolute(self.get_argument(mem)), 4),
			0x3d => (AND, AbsoluteIndexedWithX(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0x39 => (AND, AbsoluteIndexedWithY(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0x21 => (AND, ZeroPageIndexedWithXIndirect(self.get_argument(mem)), 6),
			0x31 => (AND, ZeroPageIndirectIndexedWithY(self.get_argument(mem)), 5),		// FIXME: +1 cycle if page crossed
			0x49 => (EOR, Immediate(self.get_argument(mem)), 2),
			0x45 => (EOR, ZeroPage(self.get_argument(mem)), 3),
			0x55 => (EOR, ZeroPageIndexedWithX(self.get_argument(mem)), 4),
			0x4d => (EOR, Absolute(self.get_argument(mem)), 4),
			0x5d => (EOR, AbsoluteIndexedWithX(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0x59 => (EOR, AbsoluteIndexedWithY(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0x41 => (EOR, ZeroPageIndexedWithXIndirect(self.get_argument(mem)), 6),
			0x51 => (EOR, ZeroPageIndirectIndexedWithY(self.get_argument(mem)), 5),		// FIXME: +1 cycle if page crossed
			0x09 => (ORA, Immediate(self.get_argument(mem)), 2),
			0x05 => (ORA, ZeroPage(self.get_argument(mem)), 3),
			0x15 => (ORA, ZeroPageIndexedWithX(self.get_argument(mem)), 4),
			0x0d => (ORA, Absolute(self.get_argument(mem)), 4),
			0x1d => (ORA, AbsoluteIndexedWithX(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0x19 => (ORA, AbsoluteIndexedWithY(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0x01 => (ORA, ZeroPageIndexedWithXIndirect(self.get_argument(mem)), 6),
			0x11 => (ORA, ZeroPageIndirectIndexedWithY(self.get_argument(mem)), 5),		// FIXME: +1 cycle if page crossed
			0x24 => (BIT, ZeroPage(self.get_argument(mem)), 3),
			0x2c => (BIT, Absolute(self.get_argument(mem)), 4),
			// Arithmetic
			0x69 => (ADC, Immediate(self.get_argument(mem)), 2),
			0x65 => (ADC, ZeroPage(self.get_argument(mem)), 3),
			0x75 => (ADC, ZeroPageIndexedWithX(self.get_argument(mem)), 4),
			0x6d => (ADC, Absolute(self.get_argument(mem)), 4),
			0x7d => (ADC, AbsoluteIndexedWithX(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0x79 => (ADC, AbsoluteIndexedWithY(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0x61 => (ADC, ZeroPageIndexedWithXIndirect(self.get_argument(mem)), 6),
			0x71 => (ADC, ZeroPageIndirectIndexedWithY(self.get_argument(mem)), 5),		// FIXME: +1 cycle if page crossed
			0xe9 => (SBC, Immediate(self.get_argument(mem)), 2),
			0xe5 => (SBC, ZeroPage(self.get_argument(mem)), 3),
			0xf5 => (SBC, ZeroPageIndexedWithX(self.get_argument(mem)), 4),
			0xed => (SBC, Absolute(self.get_argument(mem)), 4),
			0xfd => (SBC, AbsoluteIndexedWithX(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0xf9 => (SBC, AbsoluteIndexedWithY(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0xe1 => (SBC, ZeroPageIndexedWithXIndirect(self.get_argument(mem)), 6),
			0xf1 => (SBC, ZeroPageIndirectIndexedWithY(self.get_argument(mem)), 5),		// FIXME: +1 cycle if page crossed
			0xc9 => (CMP, Immediate(self.get_argument(mem)), 2),
			0xc5 => (CMP, ZeroPage(self.get_argument(mem)), 3),
			0xd5 => (CMP, ZeroPageIndexedWithX(self.get_argument(mem)), 4),
			0xcd => (CMP, Absolute(self.get_argument(mem)), 4),
			0xdd => (CMP, AbsoluteIndexedWithX(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0xd9 => (CMP, AbsoluteIndexedWithY(self.get_argument(mem)), 4),				// FIXME: +1 cycle if page crossed
			0xc1 => (CMP, ZeroPageIndexedWithXIndirect(self.get_argument(mem)), 6),
			0xd1 => (CMP, ZeroPageIndirectIndexedWithY(self.get_argument(mem)), 5),		// FIXME: +1 cycle if page crossed
			0xe0 => (CPX, Immediate(self.get_argument(mem)), 2),
			0xe4 => (CPX, ZeroPage(self.get_argument(mem)), 3),
			0xec => (CPX, Absolute(self.get_argument(mem)), 4),
			0xc0 => (CPY, Immediate(self.get_argument(mem)), 2),
			0xc4 => (CPY, ZeroPage(self.get_argument(mem)), 3),
			0xcc => (CPY, Absolute(self.get_argument(mem)), 4),
			// Increments & decrements
			0xe6 => (INC, ZeroPage(self.get_argument(mem)), 5),
			0xf6 => (INC, ZeroPageIndexedWithX(self.get_argument(mem)), 6),
			0xee => (INC, Absolute(self.get_argument(mem)), 6),
			0xfe => (INC, AbsoluteIndexedWithX(self.get_argument(mem)), 7),
			0xe8 => (INX, Implied, 2),
			0xc8 => (INY, Implied, 2),
			0xc6 => (DEC, ZeroPage(self.get_argument(mem)), 5),
			0xd6 => (DEC, ZeroPageIndexedWithX(self.get_argument(mem)), 6),
			0xce => (DEC, Absolute(self.get_argument(mem)), 6),
			0xde => (DEC, AbsoluteIndexedWithX(self.get_argument(mem)), 7),
			0xca => (DEX, Implied, 2),
			0x88 => (DEY, Implied, 2),
			// Shifts
			0x0a => (ASL, Accumulator, 2),
			0x06 => (ASL, ZeroPage(self.get_argument(mem)), 5),
			0x16 => (ASL, ZeroPageIndexedWithX(self.get_argument(mem)), 6),
			0x0e => (ASL, Absolute(self.get_argument(mem)), 6),
			0x1e => (ASL, AbsoluteIndexedWithX(self.get_argument(mem)), 7),
			0x4a => (LSR, Accumulator, 2),
			0x46 => (LSR, ZeroPage(self.get_argument(mem)), 5),
			0x56 => (LSR, ZeroPageIndexedWithX(self.get_argument(mem)), 6),
			0x4e => (LSR, Absolute(self.get_argument(mem)), 6),
			0x5e => (LSR, AbsoluteIndexedWithX(self.get_argument(mem)), 7),
			0x2a => (ROL, Accumulator, 2),
			0x26 => (ROL, ZeroPage(self.get_argument(mem)), 5),
			0x36 => (ROL, ZeroPageIndexedWithX(self.get_argument(mem)), 6),
			0x2e => (ROL, Absolute(self.get_argument(mem)), 6),
			0x3e => (ROL, AbsoluteIndexedWithX(self.get_argument(mem)), 7),
			0x6a => (ROR, Accumulator, 2),
			0x66 => (ROR, ZeroPage(self.get_argument(mem)), 5),
			0x76 => (ROR, ZeroPageIndexedWithX(self.get_argument(mem)), 6),
			0x6e => (ROR, Absolute(self.get_argument(mem)), 6),
			0x7e => (ROR, AbsoluteIndexedWithX(self.get_argument(mem)), 7),
			// Jump & calls
			0x4c => (JMP, Absolute(self.get_argument(mem)), 3),
			0x6c => (JMP, Indirect(self.get_argument(mem)), 5),
			0x20 => (JSR, Absolute(self.get_argument(mem)), 6),
			0x60 => (RTS, Implied, 6),
			// Branches
			0x90 => (BCC, Relative(self.get_argument(mem)), 2),					// FIXME: +1 if branched, +2 if page crossed
			0xb0 => (BCS, Relative(self.get_argument(mem)), 2),					// FIXME: +1 if branched, +2 if page crossed
			0xf0 => (BEQ, Relative(self.get_argument(mem)), 2),					// FIXME: +1 if branched, +2 if page crossed
			0x30 => (BMI, Relative(self.get_argument(mem)), 2),					// FIXME: +1 if branched, +2 if page crossed
			0xd0 => (BNE, Relative(self.get_argument(mem)), 2),					// FIXME: +1 if branched, +2 if page crossed
			0x10 => (BPL, Relative(self.get_argument(mem)), 2),					// FIXME: +1 if branched, +2 if page crossed
			0x50 => (BVC, Relative(self.get_argument(mem)), 2),					// FIXME: +1 if branched, +2 if page crossed
			0x70 => (BVS, Relative(self.get_argument(mem)), 2),					// FIXME: +1 if branched, +2 if page crossed
			// Status flag changes
			0x18 => (CLC, Implied, 2),
			0xd8 => (CLD, Implied, 2),
			0x58 => (CLI, Implied, 2),
			0xb8 => (CLV, Implied, 2),
			0x38 => (SEC, Implied, 2),
			0xf8 => (SED, Implied, 2),
			0x78 => (SEI, Implied, 2),
			// System functions
			0x00 => (BRK, Implied, 7),
			0xea => (NOP, Implied, 2),
			0x40 => (RTI, Implied, 6),
			// Illegal instruction
			opcode => fail!("mos65xx: Illegal instruction $%02x at $%04x", opcode as uint, self.pc as uint),
		};
		instruction.execute(&operand, self, mem);
		debug!("mos65xx: %04X  %02X %02X %02X  %-3s %-15s  -(%u)-> AC:%02X X:%02X Y:%02X SR:%02X SP:%02X NV-BDIZC:%8s", old_pc as uint, mem.get(old_pc) as uint, mem.get(old_pc+1) as uint, mem.get(old_pc+2) as uint, instruction.as_str(), operand.as_str(), cycles, self.ac as uint, self.x as uint, self.y as uint, self.sr as uint, self.sp as uint, self.sr.to_str_radix(2));
		cycles
	}
}


pub struct Mos6510 {
	priv cpu: Mos6502,						// Core CPU
	priv port_ddr: u8,						// CPU port data direction register
	priv port_dat: u8,						// CPU port data register
}

impl Mos6510 {
	pub fn new () -> Mos6510 {
		// TODO: addresses $0000 (data direction) and $0001 (data) are hardwired for the processor I/O port
		Mos6510 { cpu: Mos6502::new(), port_ddr: 0, port_dat: 0 }
	}

	pub fn reset<M: Addressable<u16>> (&mut self, mem: &M) {
		self.cpu.reset(mem);
	}

	pub fn step<M: Addressable<u16>> (&mut self, mem: &mut M) -> uint {
		self.cpu.step(mem)
	}
}


#[test]
fn test_addressing_modes () {
	let mut cpu = Mos6502 { pc: 0x1337, ac: 0x88, x: 0x11, y: 0x22, sr: 0, sp: 0 };
	let mut mem = TestMemory::new::<u16>();
	// Immediate
	assert_eq!(Immediate(0x55).get(&cpu, &mem), 0x55);
	// Accumulator
	assert_eq!(Accumulator.get(&cpu, &mem), 0x88);
	Accumulator.set(&mut cpu, &mut mem, 0x99); assert_eq!(cpu.ac, 0x99);
	// Relative
	assert_eq!(Relative(0x33).addr(&cpu, &mem), 0x136a);
	assert_eq!(Relative(0x99).addr(&cpu, &mem), 0x12d0);
	// Absolute
	assert_eq!(Absolute(0x0123).addr(&cpu, &mem), 0x0123);
	assert_eq!(Absolute(0x0123).get(&cpu, &mem), 0x24);
	Absolute(0x0123).set(&mut cpu, &mut mem, 0x24);
	// AbsoluteIndexedWithX
	assert_eq!(AbsoluteIndexedWithX(0x0123).addr(&cpu, &mem), 0x0134);
	assert_eq!(AbsoluteIndexedWithX(0x0123).get(&cpu, &mem), 0x35);
	AbsoluteIndexedWithX(0x0123).set(&mut cpu, &mut mem, 0x35);
	// AbsoluteIndexedWithY
	assert_eq!(AbsoluteIndexedWithY(0x0123).addr(&cpu, &mem), 0x0145);
	assert_eq!(AbsoluteIndexedWithY(0x0123).get(&cpu, &mem), 0x46);
	AbsoluteIndexedWithY(0x0123).set(&mut cpu, &mut mem, 0x46);
	// Indirect
	assert_eq!(Indirect(0x0123).addr(&cpu, &mem), 0x2524);
	assert_eq!(Indirect(0x0123).get(&cpu, &mem), 0x49);
	Indirect(0x0123).set(&mut cpu, &mut mem, 0x49);
	// ZeroPage
	assert_eq!(ZeroPage(0x12).addr(&cpu, &mem), 0x0012);
	assert_eq!(ZeroPage(0x12).get(&cpu, &mem), 0x12);
	ZeroPage(0x12).set(&mut cpu, &mut mem, 0x12);
	// ZeroPageIndexedWithX
	assert_eq!(ZeroPageIndexedWithX(0x12).addr(&cpu, &mem), 0x0023);
	assert_eq!(ZeroPageIndexedWithX(0x12).get(&cpu, &mem), 0x23);
	ZeroPageIndexedWithX(0x12).set(&mut cpu, &mut mem, 0x23);
	// ZeroPageIndexedWithY
	assert_eq!(ZeroPageIndexedWithY(0x12).addr(&cpu, &mem), 0x0034);
	assert_eq!(ZeroPageIndexedWithY(0x12).get(&cpu, &mem), 0x34);
	ZeroPageIndexedWithY(0x12).set(&mut cpu, &mut mem, 0x34);
	// ZeroPageIndexedWithXIndirect
	assert_eq!(ZeroPageIndexedWithXIndirect(0x12).addr(&cpu, &mem), 0x2423);
	assert_eq!(ZeroPageIndexedWithXIndirect(0x12).get(&cpu, &mem), 0x47);
	ZeroPageIndexedWithXIndirect(0x12).set(&mut cpu, &mut mem, 0x47);
	// ZeroPageIndirectIndexedWithY
	assert_eq!(ZeroPageIndirectIndexedWithY(0x12).addr(&cpu, &mem), 0x1334);
	assert_eq!(ZeroPageIndirectIndexedWithY(0x12).get(&cpu, &mem), 0x47);
	ZeroPageIndirectIndexedWithY(0x12).set(&mut cpu, &mut mem, 0x47);
}

#[test]
fn test_indirect_addressing_bug () {
	let cpu = Mos6502 { pc: 0x1337, ac: 0x88, x: 0x11, y: 0x22, sr: 0, sp: 0 };
	let mem = TestMemory::new::<u16>();
	// Indirect jump to ($c0ff) will get erroneously address from $c0ff/$c000 instead of $c0ff/$c100
	assert_eq!(Indirect(0xc0ff).addr(&cpu, &mem), 0xc0bf);	// must be $c0bf instead of $c1bf
}
