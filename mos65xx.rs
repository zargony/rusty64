use addressable::Addressable;
use addressable::AddressableUtil;
use ram::Ram;

static NMI_VECTOR: u16 = 0xfffa;
static RESET_VECTOR: u16 = 0xfffc;
static IRQ_VECTOR: u16 = 0xfffe;

struct Registers {
	pc: u16,				// program counter
	ac: u8,					// accumulator
	x: u8,					// x register
	y: u8,					// y register
	sr: u8,					// status register (NV-BDIZC: Negative, oVerflow, Break, Decimal, Interrupt, Zero, Carry)
	sp: u8,					// stack pointer
}

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
	fn addr (&self, cpu: &Mos6502) -> u16 {
		match * self {
			Implied								=> fail!("mos6510: Implied operand is never targetted to an address"),
			Immediate(_)						=> fail!("mos6510: Immediade operand is never targetted to an address"),
			Accumulator							=> fail!("mos6510: Accumulator operand is never targetted to an address"),
			Relative(offset)					=> cpu.reg.pc + offset as u16,
			Absolute(addr)						=> addr,
			AbsoluteIndexedWithX(addr)			=> addr + cpu.reg.x as u16,
			AbsoluteIndexedWithY(addr)			=> addr + cpu.reg.y as u16,
			Indirect(addr)						=> cpu.get_le(addr),
			ZeroPage(addr)						=> addr as u16,
			ZeroPageIndexedWithX(addr)			=> (addr + cpu.reg.x) as u16,				// no page transition
			ZeroPageIndexedWithY(addr)			=> (addr + cpu.reg.y) as u16,				// no page transition
			ZeroPageIndexedWithXIndirect(addr)	=> cpu.get_le((addr + cpu.reg.x) as u16),	// no page transition
			ZeroPageIndirectIndexedWithY(addr)	=> { let iaddr: u16 = cpu.get_le(addr as u16); iaddr + cpu.reg.y as u16 },
		}
	}

	fn get (&self, cpu: &Mos6502) -> u8 {
		match *self {
			Implied								=> fail!("mos6510: Implied operand never has a value"),
			Immediate(val)						=> val,
			Accumulator							=> cpu.reg.ac,
			Relative(_target)					=> fail!("mos6510: Relative operand never has a value"),
			op									=> { let addr = op.addr(cpu); cpu.get(addr) },
		}
	}

	fn set (&self, cpu: &mut Mos6502, val: u8) {
		match *self {
			Implied								=> fail!("mos6510: Implied operand never sets a value"),
			Immediate(_)						=> fail!("mos6510: Immediate operand never sets a value"),
			Accumulator							=> cpu.reg.ac = val,
			Relative(_target)					=> fail!("mos6510: Relative operand never sets a value"),
			op									=> { let addr = op.addr(cpu); cpu.set(addr, val); },
		}
	}
}


pub struct Mos6502<'self> {
	priv reg: Registers,		// internal CPU registers
	priv mem: Ram<u16>,			// memory as accessible to the CPU
}

impl<'self> Mos6502<'self> {
	pub fn new () -> Mos6502 {
		Mos6502 {
			reg: Registers { pc: 0, ac: 0, x: 0, y: 0, sr: 0, sp: 0 },
			mem: Ram::new()
		}
	}
}

impl<'self> Addressable<u16> for Mos6502<'self> {
	pub fn get (&self, addr: u16) -> u8 {
		// TODO: addresses $0000 (data direction) and $0001 (data) are hardwired for the processor I/O port
		self.mem.get(addr)
	}

	pub fn set (&mut self, addr: u16, data: u8) {
		// TODO: addresses $0000 (data direction) and $0001 (data) are hardwired for the processor I/O port
		self.mem.set(addr, data);
	}
}


pub struct Mos6510<'self> {
	priv cpu: Mos6502<'self>,	// Core CPU
	priv port_ddr: u8,			// CPU port data direction register
	priv port_dat: u8,			// CPU port data register
}

impl<'self> Mos6510<'self> {
	pub fn new () -> Mos6510 {
		Mos6510 { cpu: Mos6502::new(), port_ddr: 0, port_dat: 0 }
	}
}
