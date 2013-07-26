use addressable::Addressable;
use addressable::AddressableUtil;

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
				cpu.set_zn(cpu.sp);
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
				fail!("mos65xx: JSR instruction not implemented yet");				// TODO
			},
			RTS => {
				fail!("mos65xx: RTS instruction not implemented yet");				// TODO
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

	fn get_opcode<M: Addressable<u16>> (&self, mem: &M) -> u8 {
		mem.get(self.pc)
	}

	fn get_argument<M: Addressable<u16>, T: Int> (&self, mem: &M) -> T {
		mem.get_le(self.pc + 1)
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

	pub fn reset<M: Addressable<u16>> (&mut self, mem: &M) {
		// On reset, the interrupt-disable flag is set (and the decimal flag is cleared in the CMOS version 65c02).
		// The other bits and all registers (including the stack pointer are unspecified and might contain random values.
		// Execution begins at the address pointed to by the reset vector at address $FFFC.
		debug!("mos65xx: Reset");
		self.pc = mem.get_le(RESET_VECTOR);
		self.sr = 0x24;
	}

	pub fn step<M: Addressable<u16>> (&mut self, mem: &mut M) -> uint {
		// TODO
		0
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
