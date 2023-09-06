//! MOS 6502
//!
//! General information on 65xx: http://en.wikipedia.org/wiki/MOS_Technology_6510
//! Useful emulator information: http://emudocs.org/?page=CPU%2065xx
//! Web simulator and much info: http://e-tradition.net/bytes/6502/
//! Good reference and overview: http://www.obelisk.demon.co.uk/index.html
//! Processor bugs and caveats : http://www.textfiles.com/apple/6502.bugs.txt
//! Emulator and test resources: http://www.6502.org/tools/emu/
//!
//! Test ROMs: http://wiki.nesdev.com/w/index.php/Emulator_tests#CPU
//!            http://www.6502.org/tools/emu/
//!            http://visual6502.org/wiki/index.php?title=6502TestPrograms
//!            http://forum.6502.org/viewtopic.php?f=2&t=2241

mod instruction;
mod operand;

use super::CPU;
use crate::addr::{Address, Integer, Masked};
use crate::mem::Addressable;
use bitflags::bitflags;
use log::{debug, trace};
use std::mem;

pub use self::instruction::Instruction;
pub use self::operand::Operand;

/// Hard-coded address where to look for the address to jump to on nonmaskable interrupt
pub const NMI_VECTOR: u16 = 0xfffa;
/// Hard-coded address where to look for the address to jump to on reset
pub const RESET_VECTOR: u16 = 0xfffc;
/// Hard-coded address where to look for the address to jump to on interrupt
pub const IRQ_VECTOR: u16 = 0xfffe;

/// The MOS6502 processor
#[derive(Debug)]
pub struct Mos6502<M> {
    pc: u16,         // Program Counter
    ac: u8,          // Accumulator
    x: u8,           // X register
    y: u8,           // Y register
    sr: StatusFlags, // Status Register
    sp: u8,          // Stack Pointer
    mem: M,          // main memory
    reset: bool,     // RESET line
    nmi: bool,       // NMI line
    irq: bool,       // IRQ line
}

bitflags! {
    /// The MOS6502 status flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StatusFlags: u8 {
        const CARRY_FLAG             = 1 << 0;  // C
        const ZERO_FLAG              = 1 << 1;  // Z
        const INTERRUPT_DISABLE_FLAG = 1 << 2;  // I
        const DECIMAL_FLAG           = 1 << 3;  // D
        const BREAK_FLAG             = 1 << 4;  // B
        const UNUSED_ALWAYS_ON_FLAG  = 1 << 5;  // -
        const OVERFLOW_FLAG          = 1 << 6;  // V
        const NEGATIVE_FLAG          = 1 << 7;  // N
    }
}

impl<M: Addressable> Mos6502<M> {
    /// Create a new MOS6502 processor
    pub fn new(mem: M) -> Mos6502<M> {
        Mos6502 {
            pc: 0x0000,
            ac: 0x00,
            x: 0x00,
            y: 0x00,
            sr: StatusFlags::UNUSED_ALWAYS_ON_FLAG,
            sp: 0x00,
            mem: mem,
            reset: true,
            nmi: false,
            irq: false,
        }
    }

    /// Get the memory contents at the current PC and advance the PC
    fn next<const N: usize, T: Integer<N>>(&mut self) -> T {
        let value = self.mem.get_le(self.pc);
        self.pc += mem::size_of::<T>() as u16;
        value
    }

    /// Parse next instruction and advance PC. Returns number of cycles, instruction and operand
    #[rustfmt::skip]
    fn next_instruction(&mut self) -> Option<(usize, Instruction, Operand)> {
        let opcode: u8 = self.next();
        Some(match opcode {
            0x00 => (7, Instruction::BRK, Operand::Implied),
            0x01 => (6, Instruction::ORA, Operand::ZeroPageIndexedWithXIndirect(self.next())),
            0x05 => (3, Instruction::ORA, Operand::ZeroPage(self.next())),
            0x06 => (5, Instruction::ASL, Operand::ZeroPage(self.next())),
            0x08 => (3, Instruction::PHP, Operand::Implied),
            0x09 => (2, Instruction::ORA, Operand::Immediate(self.next())),
            0x0a => (2, Instruction::ASL, Operand::Accumulator),
            0x0d => (4, Instruction::ORA, Operand::Absolute(self.next())),
            0x0e => (6, Instruction::ASL, Operand::Absolute(self.next())),
            0x10 => (2, Instruction::BPL, Operand::Relative(self.next())), // +1 cycle if branched, +2 if page crossed
            0x11 => (5, Instruction::ORA, Operand::ZeroPageIndirectIndexedWithY(self.next())), // +1 cycle if page crossed
            0x15 => (4, Instruction::ORA, Operand::ZeroPageIndexedWithX(self.next())),
            0x16 => (6, Instruction::ASL, Operand::ZeroPageIndexedWithX(self.next())),
            0x18 => (2, Instruction::CLC, Operand::Implied),
            0x19 => (4, Instruction::ORA, Operand::AbsoluteIndexedWithY(self.next())), // +1 cycle if page crossed
            0x1d => (4, Instruction::ORA, Operand::AbsoluteIndexedWithX(self.next())), // +1 cycle if page crossed
            0x1e => (7, Instruction::ASL, Operand::AbsoluteIndexedWithX(self.next())),
            0x20 => (6, Instruction::JSR, Operand::Absolute(self.next())),
            0x21 => (6, Instruction::AND, Operand::ZeroPageIndexedWithXIndirect(self.next())),
            0x24 => (3, Instruction::BIT, Operand::ZeroPage(self.next())),
            0x25 => (3, Instruction::AND, Operand::ZeroPage(self.next())),
            0x26 => (5, Instruction::ROL, Operand::ZeroPage(self.next())),
            0x28 => (4, Instruction::PLP, Operand::Implied),
            0x29 => (2, Instruction::AND, Operand::Immediate(self.next())),
            0x2a => (2, Instruction::ROL, Operand::Accumulator),
            0x2c => (4, Instruction::BIT, Operand::Absolute(self.next())),
            0x2d => (4, Instruction::AND, Operand::Absolute(self.next())),
            0x2e => (6, Instruction::ROL, Operand::Absolute(self.next())),
            0x30 => (2, Instruction::BMI, Operand::Relative(self.next())), // +1 cycle if branched, +2 if page crossed
            0x31 => (5, Instruction::AND, Operand::ZeroPageIndirectIndexedWithY(self.next())), // +1 cycle if page crossed
            0x35 => (4, Instruction::AND, Operand::ZeroPageIndexedWithX(self.next())),
            0x36 => (6, Instruction::ROL, Operand::ZeroPageIndexedWithX(self.next())),
            0x38 => (2, Instruction::SEC, Operand::Implied),
            0x39 => (4, Instruction::AND, Operand::AbsoluteIndexedWithY(self.next())), // +1 cycle if page crossed
            0x3d => (4, Instruction::AND, Operand::AbsoluteIndexedWithX(self.next())), // +1 cycle if page crossed
            0x3e => (7, Instruction::ROL, Operand::AbsoluteIndexedWithX(self.next())),
            0x40 => (6, Instruction::RTI, Operand::Implied),
            0x41 => (6, Instruction::EOR, Operand::ZeroPageIndexedWithXIndirect(self.next())),
            0x45 => (3, Instruction::EOR, Operand::ZeroPage(self.next())),
            0x46 => (5, Instruction::LSR, Operand::ZeroPage(self.next())),
            0x48 => (3, Instruction::PHA, Operand::Implied),
            0x49 => (2, Instruction::EOR, Operand::Immediate(self.next())),
            0x4a => (2, Instruction::LSR, Operand::Accumulator),
            0x4c => (3, Instruction::JMP, Operand::Absolute(self.next())),
            0x4d => (4, Instruction::EOR, Operand::Absolute(self.next())),
            0x4e => (6, Instruction::LSR, Operand::Absolute(self.next())),
            0x50 => (2, Instruction::BVC, Operand::Relative(self.next())), // +1 cycle if branched, +2 if page crossed
            0x51 => (5, Instruction::EOR, Operand::ZeroPageIndirectIndexedWithY(self.next())), // +1 cycle if page crossed
            0x55 => (4, Instruction::EOR, Operand::ZeroPageIndexedWithX(self.next())),
            0x56 => (6, Instruction::LSR, Operand::ZeroPageIndexedWithX(self.next())),
            0x58 => (2, Instruction::CLI, Operand::Implied),
            0x59 => (4, Instruction::EOR, Operand::AbsoluteIndexedWithY(self.next())), // +1 cycle if page crossed
            0x5d => (4, Instruction::EOR, Operand::AbsoluteIndexedWithX(self.next())), // +1 cycle if page crossed
            0x5e => (7, Instruction::LSR, Operand::AbsoluteIndexedWithX(self.next())),
            0x60 => (6, Instruction::RTS, Operand::Implied),
            0x61 => (6, Instruction::ADC, Operand::ZeroPageIndexedWithXIndirect(self.next())),
            0x65 => (3, Instruction::ADC, Operand::ZeroPage(self.next())),
            0x66 => (5, Instruction::ROR, Operand::ZeroPage(self.next())),
            0x68 => (4, Instruction::PLA, Operand::Implied),
            0x69 => (2, Instruction::ADC, Operand::Immediate(self.next())),
            0x6a => (2, Instruction::ROR, Operand::Accumulator),
            0x6c => (5, Instruction::JMP, Operand::Indirect(self.next())),
            0x6d => (4, Instruction::ADC, Operand::Absolute(self.next())),
            0x6e => (6, Instruction::ROR, Operand::Absolute(self.next())),
            0x70 => (2, Instruction::BVS, Operand::Relative(self.next())), // +1 cycle if branched, +2 if page crossed
            0x71 => (5, Instruction::ADC, Operand::ZeroPageIndirectIndexedWithY(self.next())), // +1 cycle if page crossed
            0x75 => (4, Instruction::ADC, Operand::ZeroPageIndexedWithX(self.next())),
            0x76 => (6, Instruction::ROR, Operand::ZeroPageIndexedWithX(self.next())),
            0x78 => (2, Instruction::SEI, Operand::Implied),
            0x79 => (4, Instruction::ADC, Operand::AbsoluteIndexedWithY(self.next())), // +1 cycle if page crossed
            0x7d => (4, Instruction::ADC, Operand::AbsoluteIndexedWithX(self.next())), // +1 cycle if page crossed
            0x7e => (7, Instruction::ROR, Operand::AbsoluteIndexedWithX(self.next())),
            0x81 => (6, Instruction::STA, Operand::ZeroPageIndexedWithXIndirect(self.next())),
            0x84 => (3, Instruction::STY, Operand::ZeroPage(self.next())),
            0x85 => (3, Instruction::STA, Operand::ZeroPage(self.next())),
            0x86 => (3, Instruction::STX, Operand::ZeroPage(self.next())),
            0x88 => (2, Instruction::DEY, Operand::Implied),
            0x8a => (2, Instruction::TXA, Operand::Implied),
            0x8c => (4, Instruction::STY, Operand::Absolute(self.next())),
            0x8d => (4, Instruction::STA, Operand::Absolute(self.next())),
            0x8e => (4, Instruction::STX, Operand::Absolute(self.next())),
            0x90 => (2, Instruction::BCC, Operand::Relative(self.next())), // +1 cycle if branched, +2 if page crossed
            0x91 => (6, Instruction::STA, Operand::ZeroPageIndirectIndexedWithY(self.next())),
            0x94 => (4, Instruction::STY, Operand::ZeroPageIndexedWithX(self.next())),
            0x95 => (4, Instruction::STA, Operand::ZeroPageIndexedWithX(self.next())),
            0x96 => (4, Instruction::STX, Operand::ZeroPageIndexedWithY(self.next())),
            0x98 => (2, Instruction::TYA, Operand::Implied),
            0x99 => (5, Instruction::STA, Operand::AbsoluteIndexedWithY(self.next())),
            0x9a => (2, Instruction::TXS, Operand::Implied),
            0x9d => (5, Instruction::STA, Operand::AbsoluteIndexedWithX(self.next())),
            0xa0 => (2, Instruction::LDY, Operand::Immediate(self.next())),
            0xa1 => (6, Instruction::LDA, Operand::ZeroPageIndexedWithXIndirect(self.next())),
            0xa2 => (2, Instruction::LDX, Operand::Immediate(self.next())),
            0xa4 => (3, Instruction::LDY, Operand::ZeroPage(self.next())),
            0xa5 => (3, Instruction::LDA, Operand::ZeroPage(self.next())),
            0xa6 => (3, Instruction::LDX, Operand::ZeroPage(self.next())),
            0xa8 => (2, Instruction::TAY, Operand::Implied),
            0xa9 => (2, Instruction::LDA, Operand::Immediate(self.next())),
            0xaa => (2, Instruction::TAX, Operand::Implied),
            0xac => (4, Instruction::LDY, Operand::Absolute(self.next())),
            0xad => (4, Instruction::LDA, Operand::Absolute(self.next())),
            0xae => (4, Instruction::LDX, Operand::Absolute(self.next())),
            0xb0 => (2, Instruction::BCS, Operand::Relative(self.next())), // +1 cycle if branched, +2 if page crossed
            0xb1 => (5, Instruction::LDA, Operand::ZeroPageIndirectIndexedWithY(self.next())), // +1 cycle if page crossed
            0xb4 => (4, Instruction::LDY, Operand::ZeroPageIndexedWithX(self.next())),
            0xb5 => (4, Instruction::LDA, Operand::ZeroPageIndexedWithX(self.next())),
            0xb6 => (4, Instruction::LDX, Operand::ZeroPageIndexedWithY(self.next())),
            0xb8 => (2, Instruction::CLV, Operand::Implied),
            0xb9 => (4, Instruction::LDA, Operand::AbsoluteIndexedWithY(self.next())), // +1 cycle if page crossed
            0xba => (2, Instruction::TSX, Operand::Implied),
            0xbc => (4, Instruction::LDY, Operand::AbsoluteIndexedWithX(self.next())), // +1 cycle if page crossed
            0xbd => (4, Instruction::LDA, Operand::AbsoluteIndexedWithX(self.next())), // +1 cycle if page crossed
            0xbe => (4, Instruction::LDX, Operand::AbsoluteIndexedWithY(self.next())), // +1 cycle if page crossed
            0xc0 => (2, Instruction::CPY, Operand::Immediate(self.next())),
            0xc1 => (6, Instruction::CMP, Operand::ZeroPageIndexedWithXIndirect(self.next())),
            0xc4 => (3, Instruction::CPY, Operand::ZeroPage(self.next())),
            0xc5 => (3, Instruction::CMP, Operand::ZeroPage(self.next())),
            0xc6 => (5, Instruction::DEC, Operand::ZeroPage(self.next())),
            0xc8 => (2, Instruction::INY, Operand::Implied),
            0xc9 => (2, Instruction::CMP, Operand::Immediate(self.next())),
            0xca => (2, Instruction::DEX, Operand::Implied),
            0xcc => (4, Instruction::CPY, Operand::Absolute(self.next())),
            0xcd => (4, Instruction::CMP, Operand::Absolute(self.next())),
            0xce => (6, Instruction::DEC, Operand::Absolute(self.next())),
            0xd0 => (2, Instruction::BNE, Operand::Relative(self.next())), // +1 cycle if branched, +2 if page crossed
            0xd1 => (5, Instruction::CMP, Operand::ZeroPageIndirectIndexedWithY(self.next())), // +1 cycle if page crossed
            0xd5 => (4, Instruction::CMP, Operand::ZeroPageIndexedWithX(self.next())),
            0xd6 => (6, Instruction::DEC, Operand::ZeroPageIndexedWithX(self.next())),
            0xd8 => (2, Instruction::CLD, Operand::Implied),
            0xd9 => (4, Instruction::CMP, Operand::AbsoluteIndexedWithY(self.next())), // +1 cycle if page crossed
            0xdd => (4, Instruction::CMP, Operand::AbsoluteIndexedWithX(self.next())), // +1 cycle if page crossed
            0xde => (7, Instruction::DEC, Operand::AbsoluteIndexedWithX(self.next())),
            0xe0 => (2, Instruction::CPX, Operand::Immediate(self.next())),
            0xe1 => (6, Instruction::SBC, Operand::ZeroPageIndexedWithXIndirect(self.next())),
            0xe4 => (3, Instruction::CPX, Operand::ZeroPage(self.next())),
            0xe5 => (3, Instruction::SBC, Operand::ZeroPage(self.next())),
            0xe6 => (5, Instruction::INC, Operand::ZeroPage(self.next())),
            0xe8 => (2, Instruction::INX, Operand::Implied),
            0xe9 => (2, Instruction::SBC, Operand::Immediate(self.next())),
            0xea => (2, Instruction::NOP, Operand::Implied),
            0xec => (4, Instruction::CPX, Operand::Absolute(self.next())),
            0xed => (4, Instruction::SBC, Operand::Absolute(self.next())),
            0xee => (6, Instruction::INC, Operand::Absolute(self.next())),
            0xf0 => (2, Instruction::BEQ, Operand::Relative(self.next())), // +1 cycle if branched, +2 if page crossed
            0xf1 => (5, Instruction::SBC, Operand::ZeroPageIndirectIndexedWithY(self.next())), // +1 cycle if page crossed
            0xf5 => (4, Instruction::SBC, Operand::ZeroPageIndexedWithX(self.next())),
            0xf6 => (6, Instruction::INC, Operand::ZeroPageIndexedWithX(self.next())),
            0xf8 => (2, Instruction::SED, Operand::Implied),
            0xf9 => (4, Instruction::SBC, Operand::AbsoluteIndexedWithY(self.next())), // +1 cycle if page crossed
            0xfd => (4, Instruction::SBC, Operand::AbsoluteIndexedWithX(self.next())), // +1 cycle if page crossed
            0xfe => (7, Instruction::INC, Operand::AbsoluteIndexedWithX(self.next())),
            // Illegal opcode
            _ => return None,
        })
    }

    /// Set ZERO_FLAG and NEGATIVE_FLAG based on the given value
    fn set_zn(&mut self, value: u8) -> u8 {
        self.sr.set(StatusFlags::ZERO_FLAG, value == 0);
        self.sr.set(StatusFlags::NEGATIVE_FLAG, (value as i8) < 0);
        value
    }

    /// Push a value onto the stack
    fn push<const N: usize, T: Integer<N>>(&mut self, value: T) {
        // SP points to the next free stack position as $0100+SP. SP needs to be
        // initialized to #$FF by the reset code. As the stack grows, SP decreases
        // down to #$00 (i.e. stack full). Stack access never leaves the stack page!
        self.sp = self.sp.wrapping_sub(mem::size_of::<T>() as u8);
        let addr = Masked(0x0100, 0xff00).offset(self.sp as i16 + 1);
        self.mem.set_le(addr, value);
    }

    /// Pop a value from the stack
    fn pop<const N: usize, T: Integer<N>>(&mut self) -> T {
        // See push() for details
        let addr = Masked(0x0100, 0xff00).offset(self.sp as i16 + 1);
        self.sp = self.sp.wrapping_add(mem::size_of::<T>() as u8);
        self.mem.get_le(addr)
    }

    /// Interrupt the CPU (NMI)
    pub fn nmi(&mut self) {
        // Trigger the NMI line. The actual NMI processing is done in the next step().
        self.nmi = true;
    }

    /// Interrupt the CPU (IRQ)
    pub fn irq(&mut self) {
        // Trigger the IRQ line. The actual IRQ processing is done in the next step().
        self.irq = true;
    }
}

impl<M: Addressable> CPU for Mos6502<M> {
    /// Reset the CPU
    fn reset(&mut self) {
        // Trigger the RESET line. The actual RESET processing is done in the next step().
        self.reset = true;
    }

    /// Do one step (execute the next instruction). Return the number of cycles
    /// that were simulated.
    fn step(&mut self) -> usize {
        // Process RESET if line was triggered
        if self.reset {
            // A RESET jumps to the vector at RESET_VECTOR and sets INTERRUPT_DISABLE_FLAG.
            // Note that all other states and registers are unspecified and might contain
            // random values, so they need to be initialized by the reset routine.
            // See also http://6502.org/tutorials/interrupts.html
            self.sr.insert(StatusFlags::INTERRUPT_DISABLE_FLAG);
            self.pc = self.mem.get_le(RESET_VECTOR);
            self.reset = false;
            self.nmi = false;
            self.irq = false;
            debug!(
                "mos6502: RESET - Jumping to ({}) -> {}",
                RESET_VECTOR.display(),
                self.pc.display()
            );
            return 6;
        }
        // Process NMI if line was triggered
        if self.nmi {
            // An NMI pushes PC and SR to the stack and jumps to the vector at NMI_VECTOR.
            // It does NOT set the INTERRUPT_DISABLE_FLAG. Unlike JSR, it pushes the address
            // of the next instruction to the stack.
            // See also http://6502.org/tutorials/interrupts.html
            self.push(self.pc);
            self.push(self.sr.bits());
            self.pc = self.mem.get_le(NMI_VECTOR);
            self.nmi = false;
            debug!(
                "mos6502: NMI - Jumping to ({}) -> {}",
                NMI_VECTOR.display(),
                self.pc.display()
            );
            return 7;
        }
        // Process IRQ if line was triggered and interrupts are enabled
        if self.irq && !self.sr.contains(StatusFlags::INTERRUPT_DISABLE_FLAG) {
            // An IRQ pushes PC and SR to the stack, jumps to the vector at IRQ_VECTOR and
            // sets the INTERRUPT_DISABLE_FLAG. Unlike JSR, it pushes the address of the next
            // instruction to the stack. This also emulates the BRK bug where a BRK instruction
            // is ignored if an IRQ occurs simultaneously.
            // The BRK instruction does the same, but sets BREAK_FLAG (before pushing SR).
            // See also http://6502.org/tutorials/interrupts.html
            self.sr.remove(StatusFlags::BREAK_FLAG);
            if self.mem.get(self.pc) == 0x00 {
                // Simulate BRK bug
                self.pc += 1;
            }
            self.push(self.pc);
            self.push(self.sr.bits());
            self.sr.insert(StatusFlags::INTERRUPT_DISABLE_FLAG);
            self.pc = self.mem.get_le(IRQ_VECTOR);
            // FIXME: The real 6502 IRQ line is level-sensitive, not edge-sensitive!
            // FIXME: I.e. it does not stop jumping to the IRQ_VECTOR after one run,
            // FIXME: but after the hardware drops the IRQ line (which the interrupt
            // FIXME: code usually causes, but not necessary needs to cause).
            self.irq = false;
            debug!(
                "mos6502: IRQ - Jumping to ({}) -> {}",
                IRQ_VECTOR.display(),
                self.pc.display()
            );
            return 7;
        }
        // Read and parse next opcode
        let old_pc = self.pc;
        match self.next_instruction() {
            // Got valid opcode
            Some((cycles, instruction, operand)) => {
                let new_pc = self.pc;
                instruction.execute(self, &operand);
                // FIXME: formatting doesn't work!?
                trace!("mos6502: {}  {:8}  {:3} {:15}  -[{}]-> AC:{:02X} X:{:02X} Y:{:02X} SR:{:02X} SP:{:02X} NV-BDIZC:{:08b}",
                    old_pc.display(), self.mem.hexdump(old_pc..new_pc), instruction, operand,
                    cycles, self.ac, self.x, self.y, self.sr.bits(), self.sp, self.sr.bits());
                cycles
            }
            // Got illegal opcode
            None => {
                trace!(
                    "mos6502: {}  {:8}  ???",
                    old_pc.display(),
                    self.mem.hexdump(old_pc..old_pc + 2)
                );
                panic!(
                    "mos6502: Illegal opcode #${:02X} at {}",
                    self.mem.get(old_pc),
                    old_pc.display()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::test::TestMemory;
    use crate::mem::{Ram, Rom};

    #[test]
    fn smoke() {
        let mut cpu = Mos6502::new(TestMemory);
        cpu.reset();
        cpu.nmi();
        cpu.irq();
        cpu.step();
    }

    #[test]
    fn initial_state() {
        let cpu = Mos6502::new(TestMemory);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sr, StatusFlags::UNUSED_ALWAYS_ON_FLAG);
        assert!(cpu.reset);
    }

    #[test]
    fn fetch_memory_contents_and_advance_pc() {
        let mut cpu = Mos6502::new(TestMemory);
        cpu.pc = 0x0012;
        let value: u8 = cpu.next();
        assert_eq!(value, 0x12);
        let value: u8 = cpu.next();
        assert_eq!(value, 0x13);
        let value: u16 = cpu.next();
        assert_eq!(value, 0x1514);
        let value: u16 = cpu.next();
        assert_eq!(value, 0x1716);
    }

    #[test]
    fn fetch_instruction_and_advance_pc() {
        let mut cpu = Mos6502::new(TestMemory);
        cpu.pc = 0x00ad; // AD AE AF: LDA $AFAE
        let (cycles, instruction, operand) = cpu.next_instruction().unwrap();
        assert_eq!(cycles, 4);
        assert_eq!(instruction, Instruction::LDA);
        assert_eq!(operand, Operand::Absolute(0xafae));
    }

    #[test]
    fn status_flags() {
        let mut cpu = Mos6502::new(TestMemory);
        cpu.sr = StatusFlags::ZERO_FLAG
            | StatusFlags::DECIMAL_FLAG
            | StatusFlags::UNUSED_ALWAYS_ON_FLAG
            | StatusFlags::NEGATIVE_FLAG;
        cpu.sr.set(StatusFlags::CARRY_FLAG, true);
        cpu.sr.set(StatusFlags::ZERO_FLAG, false);
        cpu.sr.set(StatusFlags::OVERFLOW_FLAG, true);
        cpu.sr.set(StatusFlags::NEGATIVE_FLAG, false);
        assert_eq!(
            cpu.sr,
            StatusFlags::CARRY_FLAG
                | StatusFlags::DECIMAL_FLAG
                | StatusFlags::UNUSED_ALWAYS_ON_FLAG
                | StatusFlags::OVERFLOW_FLAG
        );
    }

    #[test]
    fn zero_and_negative_values() {
        let mut cpu = Mos6502::new(TestMemory);
        cpu.set_zn(0);
        assert!(cpu.sr.contains(StatusFlags::ZERO_FLAG));
        assert!(!cpu.sr.contains(StatusFlags::NEGATIVE_FLAG));
        cpu.set_zn(42);
        assert!(!cpu.sr.contains(StatusFlags::ZERO_FLAG));
        assert!(!cpu.sr.contains(StatusFlags::NEGATIVE_FLAG));
        cpu.set_zn(142);
        assert!(!cpu.sr.contains(StatusFlags::ZERO_FLAG));
        assert!(cpu.sr.contains(StatusFlags::NEGATIVE_FLAG));
    }

    #[test]
    fn stack_push_pop() {
        let mut cpu = Mos6502::new(Ram::with_capacity(0x01ff));
        cpu.sp = 0xff;
        cpu.push(0x12_u8);
        assert_eq!(cpu.sp, 0xfe);
        assert_eq!(cpu.mem.get(0x01ff), 0x12);
        cpu.push(0x3456_u16);
        assert_eq!(cpu.sp, 0xfc);
        assert_eq!(cpu.mem.get(0x01fe), 0x34);
        assert_eq!(cpu.mem.get(0x01fd), 0x56);
        let value: u8 = cpu.pop();
        assert_eq!(value, 0x56);
        assert_eq!(cpu.sp, 0xfd);
        let value: u16 = cpu.pop();
        assert_eq!(value, 0x1234);
        assert_eq!(cpu.sp, 0xff);
    }

    #[test]
    fn stack_overflow() {
        let mut cpu = Mos6502::new(Ram::with_capacity(0x01ff));
        cpu.sp = 0x00;
        cpu.push(0x12_u8);
        assert_eq!(cpu.sp, 0xff);
        assert_eq!(cpu.mem.get(0x0100), 0x12);
        let value: u8 = cpu.pop();
        assert_eq!(value, 0x12);
        assert_eq!(cpu.sp, 0x00);
    }

    #[test]
    fn stack_overflow_word() {
        let mut cpu = Mos6502::new(Ram::with_capacity(0x01ff));
        cpu.sp = 0x00;
        cpu.push(0x1234_u16);
        assert_eq!(cpu.sp, 0xfe);
        assert_eq!(cpu.mem.get(0x0100), 0x12);
        assert_eq!(cpu.mem.get(0x01ff), 0x34);
        let value: u16 = cpu.pop();
        assert_eq!(value, 0x1234);
        assert_eq!(cpu.sp, 0x00);
    }

    #[test]
    fn state_after_nmi() {
        let mut cpu = Mos6502::new(Ram::with_capacity(0xffff));
        cpu.sr =
            StatusFlags::CARRY_FLAG | StatusFlags::ZERO_FLAG | StatusFlags::UNUSED_ALWAYS_ON_FLAG;
        cpu.sp = 0xff;
        cpu.mem.set_le(0xfffa, 0x1234_u16);
        cpu.reset = false;
        cpu.nmi();
        cpu.step();
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(
            cpu.sr,
            StatusFlags::CARRY_FLAG | StatusFlags::ZERO_FLAG | StatusFlags::UNUSED_ALWAYS_ON_FLAG
        );
        assert_eq!(cpu.sp, 0xfc);
    }

    #[test]
    fn state_after_irq() {
        let mut cpu = Mos6502::new(Ram::with_capacity(0xffff));
        cpu.sr =
            StatusFlags::CARRY_FLAG | StatusFlags::ZERO_FLAG | StatusFlags::UNUSED_ALWAYS_ON_FLAG;
        cpu.sp = 0xff;
        cpu.mem.set_le(0xfffe, 0x1234_u16);
        cpu.reset = false;
        cpu.irq();
        cpu.step();
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(
            cpu.sr,
            StatusFlags::CARRY_FLAG
                | StatusFlags::ZERO_FLAG
                | StatusFlags::INTERRUPT_DISABLE_FLAG
                | StatusFlags::UNUSED_ALWAYS_ON_FLAG
        );
        assert_eq!(cpu.sp, 0xfc);
    }

    #[test]
    fn state_after_reset() {
        let mut cpu = Mos6502::new(Ram::with_capacity(0xffff));
        cpu.sr =
            StatusFlags::CARRY_FLAG | StatusFlags::ZERO_FLAG | StatusFlags::UNUSED_ALWAYS_ON_FLAG;
        cpu.sp = 0xff;
        cpu.mem.set_le(0xfffc, 0x1234_u16);
        cpu.reset();
        cpu.step();
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(
            cpu.sr,
            StatusFlags::CARRY_FLAG
                | StatusFlags::ZERO_FLAG
                | StatusFlags::INTERRUPT_DISABLE_FLAG
                | StatusFlags::UNUSED_ALWAYS_ON_FLAG
        );
        assert_eq!(cpu.sp, 0xff);
    }

    #[test]
    fn brk_bug() {
        let mut cpu = Mos6502::new(Ram::with_capacity(0xffff));
        cpu.pc = 0x1000;
        cpu.sr = StatusFlags::UNUSED_ALWAYS_ON_FLAG;
        cpu.sp = 0xff;
        cpu.mem.set_le(0x1000, 0x00_u8); // 00: BRK
        cpu.mem.set_le(0x2000, 0x40_u8); // 40: RTI
        cpu.mem.set_le(0xfffe, 0x2000_u16);
        cpu.reset = false;
        cpu.irq();
        cpu.step(); // IRQ happens when BRK is next instruction
        assert_eq!(cpu.pc, 0x2000); // IRQ is handled
        assert!(!cpu.sr.contains(StatusFlags::BREAK_FLAG));
        cpu.step(); // IRQ handler returns
        assert_eq!(cpu.pc, 0x1001); // BRK was skipped
    }

    #[test]
    fn ruud_baltissen_core_instruction_rom() {
        // Test all instructions using Ruud Baltissen's test ROM from his VHDL 6502 core.
        // See also http://visual6502.org/wiki/index.php?title=6502TestPrograms
        let mut cpu = Mos6502::new(Ram::with_capacity(0xffff));
        for addr in 0x0000..0xe000 {
            cpu.mem.set(addr, 0x00);
        }
        let rom = Rom::new("test/ttl6502_v10.rom");
        cpu.mem.copy(0xe000, &rom, 0x0000, rom.capacity());
        cpu.reset();
        for _ in 0..3000 {
            cpu.step();
            // TODO: This skips decimal mode tests for now
            if cpu.pc == 0xf5b6 {
                cpu.pc = 0xf5e6;
            }
        }
        let status = cpu.mem.get(0x0003);
        assert!(
            status == 0xfe,
            "stopped at {} with status #${:02X}",
            cpu.pc.display(),
            status,
        );
    }
}
