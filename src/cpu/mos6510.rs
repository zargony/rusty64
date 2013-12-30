use mem::Addressable;
use super::cpu::CPU;
use super::mos6502::Mos6502;

/// The MOS65010 processor
pub struct Mos6510 {
	priv cpu: Mos6502,						// Core CPU is a MOS6502
	priv port_ddr: u8,						// CPU port data direction register
	priv port_dat: u8,						// CPU port data register
}

impl Mos6510 {
	/// Create a new MOS6510 processor
	pub fn new () -> Mos6510 {
		// TODO: addresses $0000 (data direction) and $0001 (data) are hardwired for the processor I/O port
		Mos6510 { cpu: Mos6502::new(), port_ddr: 0, port_dat: 0 }
	}

	/// Interrupt the CPU (NMI)
	pub fn nmi (&mut self) {
		self.cpu.nmi();
	}

	/// Interrupt the CPU (IRQ)
	pub fn irq (&mut self) {
		self.cpu.irq();
	}
}

impl CPU<u16> for Mos6510 {
	/// Reset the CPU
	fn reset (&mut self) {
		self.cpu.reset();
	}

	/// Do one step (execute the next instruction). Returns the number of
	/// cycles the instruction needed
	fn step<M: Addressable<u16>> (&mut self, mem: &mut M) -> uint {
		self.cpu.step(mem)
	}
}
