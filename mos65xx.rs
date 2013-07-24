use addressable::Addressable;
use addressable::AddressableUtil;

// General information on 65xx: http://en.wikipedia.org/wiki/MOS_Technology_6510
// Web simulator and much info: http://e-tradition.net/bytes/6502/
// Good reference and overview: http://www.obelisk.demon.co.uk/index.html

static NMI_VECTOR: u16 = 0xfffa;
static RESET_VECTOR: u16 = 0xfffc;
static IRQ_VECTOR: u16 = 0xfffe;

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
	fn addr<M: Addressable<u16>> (&self, cpu: &Mos6502, mem: &M) -> u16 {
		match *self {
			Implied								=> fail!("mos65xx: Implied operand is never targetted to an address"),
			Immediate(_)						=> fail!("mos65xx: Immediate operand is never targetted to an address"),
			Accumulator							=> fail!("mos65xx: Accumulator operand is never targetted to an address"),
			Relative(offset)					=> cpu.pc + offset as u16,
			Absolute(addr)						=> addr,
			AbsoluteIndexedWithX(addr)			=> addr + cpu.x as u16,
			AbsoluteIndexedWithY(addr)			=> addr + cpu.y as u16,
			Indirect(addr)						=> mem.get_le(addr),
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
			Immediate(val)						=> val,
			Accumulator							=> cpu.ac,
			Relative(_target)					=> fail!("mos65xx: Relative operand never has a value"),
			op									=> { let addr = op.addr(cpu, mem); mem.get(addr) },
		}
	}

	fn set<M: Addressable<u16>> (&self, cpu: &mut Mos6502, mem: &mut M, val: u8) {
		match *self {
			Implied								=> fail!("mos65xx: Implied operand never sets a value"),
			Immediate(_)						=> fail!("mos65xx: Immediate operand never sets a value"),
			Accumulator							=> cpu.ac = val,
			Relative(_target)					=> fail!("mos65xx: Relative operand never sets a value"),
			op									=> { let addr = op.addr(cpu, mem); mem.set(addr, val); },
		}
	}
}


pub struct Mos6502 {
	priv pc: u16,						// program counter
	priv ac: u8,						// accumulator
	priv x: u8,							// x register
	priv y: u8,							// y register
	priv sr: u8,						// status register (NV-BDIZC: Negative, oVerflow, Break, Decimal, Interrupt, Zero, Carry)
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

	pub fn step<M: Addressable<u16>> (&mut self, mem: &mut M) -> uint {
		self.cpu.step(mem)
	}
}
