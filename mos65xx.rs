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
