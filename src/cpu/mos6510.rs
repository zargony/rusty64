//!
//! MOS 6510
//!

use mem::Addressable;
use cpu::{CPU, Mos6502};

/// The MOS65010 processor
pub struct Mos6510<M> {
    cpu: Mos6502<M>,                        // Core CPU is a MOS6502
    port_ddr: u8,                           // CPU port data direction register
    port_dat: u8,                           // CPU port data register
}

impl<M: Addressable> Mos6510<M> {
    /// Create a new MOS6510 processor
    pub fn new (mem: M) -> Mos6510<M> {
        // TODO: addresses $0000 (data direction) and $0001 (data) are hardwired for the processor I/O port
        Mos6510 { cpu: Mos6502::new(mem), port_ddr: 0, port_dat: 0 }
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

impl<M: Addressable> CPU for Mos6510<M> {
    /// Reset the CPU
    fn reset (&mut self) {
        self.cpu.reset();
    }

    /// Do one step (execute the next instruction). Return the number of cycles
    /// that were simulated.
    fn step (&mut self) -> usize {
        self.cpu.step()
    }
}


#[cfg(test)]
mod tests {
    use mem::test::TestMemory;
    use cpu::CPU;
    use super::*;

    #[test]
    fn smoke () {
        let mut cpu = Mos6510::new(TestMemory);
        cpu.reset();
        cpu.nmi();
        cpu.irq();
        cpu.step();
    }
}
