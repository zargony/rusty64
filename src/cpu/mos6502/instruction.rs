//!
//! MOS 6502 Instruction set
//!

use std::fmt;
use addr::Address;
use mem::Addressable;
use super::{Mos6502, Operand, IRQ_VECTOR};
use super::{CarryFlag, ZeroFlag, InterruptDisableFlag, DecimalFlag};
use super::{BreakFlag, UnusedAlwaysOnFlag, NegativeFlag, OverflowFlag};

/// Processor instructions
#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
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
    /// Execute an instruction using the given environment
    pub fn execute<M: Addressable> (&self, cpu: &mut Mos6502<M>, operand: &Operand) {
        match *self {
            // Load/store operations
            Instruction::LDA => {                   // load accumulator [N,Z]
                let value = operand.get(cpu);
                cpu.ac = value;
                cpu.set_zn(value);
            },
            Instruction::LDX => {                   // load X register [N,Z]
                let value = operand.get(cpu);
                cpu.x = value;
                cpu.set_zn(value);
            },
            Instruction::LDY => {                   // load Y register [N,Z]
                let value = operand.get(cpu);
                cpu.y = value;
                cpu.set_zn(value);
            },
            Instruction::STA => {                   // store accumulator
                let value = cpu.ac;
                operand.set(cpu, value);
            },
            Instruction::STX => {                   // store X register
                let value = cpu.x;
                operand.set(cpu, value);
            },
            Instruction::STY => {                   // store Y register
                let value = cpu.y;
                operand.set(cpu, value);
            },
            // Register transfers
            Instruction::TAX => {                   // transfer accumulator to X [N,Z]
                let value = cpu.ac;
                cpu.x = value;
                cpu.set_zn(value);
            },
            Instruction::TAY => {                   // transfer accumulator to Y [N,Z]
                let value = cpu.ac;
                cpu.y = value;
                cpu.set_zn(value);
            },
            Instruction::TXA => {                   // transfer X to accumulator [N,Z]
                let value = cpu.x;
                cpu.ac = value;
                cpu.set_zn(value);
            },
            Instruction::TYA => {                   // transfer Y to accumulator [N,Z]
                let value = cpu.y;
                cpu.ac = value;
                cpu.set_zn(value);
            },
            // Stack operations
            Instruction::TSX => {                   // transfer stack pointer to X [N,Z]
                let value = cpu.sp;
                cpu.x = value;
                cpu.set_zn(value);
            },
            Instruction::TXS => {                   // transfer X to stack pointer
                cpu.sp = cpu.x;
            },
            Instruction::PHA => {                   // push accumulator on stack
                let value = cpu.ac;
                cpu.push(value);
            },
            Instruction::PHP => {                   // push processor status (SR) on stack
                let value = cpu.sr.bits;
                cpu.push(value);
            },
            Instruction::PLA => {                   // pull accumulator from stack [N,Z]
                let value = cpu.pop();
                cpu.ac = value;
                cpu.set_zn(value);
            },
            Instruction::PLP => {                   // pull processor status (SR) from stack [all]
                cpu.sr.bits = cpu.pop();
                cpu.sr.insert(UnusedAlwaysOnFlag);
            },
            // Logical
            Instruction::AND => {                   // logical AND [N,Z]
                let result = cpu.ac & operand.get(cpu);
                cpu.ac = result;
                cpu.set_zn(result);
            },
            Instruction::EOR => {                   // logical exclusive OR [N,Z]
                let result = cpu.ac ^ operand.get(cpu);
                cpu.ac = result;
                cpu.set_zn(result);
            },
            Instruction::ORA => {                   // logical inclusive OR [N,Z]
                let result = cpu.ac | operand.get(cpu);
                cpu.ac = result;
                cpu.set_zn(result);
            },
            Instruction::BIT => {                   // bit test [N,V,Z]
                let value = operand.get(cpu);
                cpu.sr.set(ZeroFlag, (value & cpu.ac) == 0);
                cpu.sr.set(NegativeFlag, (value & 0x80) != 0);
                cpu.sr.set(OverflowFlag, (value & 0x40) != 0);
            },
            // Arithmetic
            Instruction::ADC => {                   // add with carry [N,V,Z,C]
                if cpu.sr.contains(DecimalFlag) { panic!("mos6502: Decimal mode ADC not supported yet :("); }
                let value = operand.get(cpu);
                let mut result = (cpu.ac as u16).wrapping_add(value as u16);
                if cpu.sr.contains(CarryFlag) { result = result.wrapping_add(1); }
                cpu.sr.set(CarryFlag, (result & 0x100) != 0);
                let result = result as u8;
                cpu.sr.set(OverflowFlag, (cpu.ac ^ value) & 0x80 == 0 && (cpu.ac ^ result) & 0x80 == 0x80);
                cpu.ac = result;
                cpu.set_zn(result);
            },
            Instruction::SBC => {                   // subtract with carry [N,V,Z,C]
                if cpu.sr.contains(DecimalFlag) { panic!("mos6502: Decimal mode ADC not supported yet :("); }
                let value = operand.get(cpu);
                let mut result = (cpu.ac as u16).wrapping_sub(value as u16);
                if !cpu.sr.contains(CarryFlag) { result = result.wrapping_sub(1); }
                cpu.sr.set(CarryFlag, (result & 0x100) == 0);
                let result = result as u8;
                cpu.sr.set(OverflowFlag, (cpu.ac ^ result) & 0x80 != 0 && (cpu.ac ^ value) & 0x80 == 0x80);
                cpu.ac = result;
                cpu.set_zn(result);
            },
            Instruction::CMP => {                   // compare (with accumulator) [N,Z,C]
                let result = cpu.ac as i16 - operand.get(cpu) as i16;
                cpu.sr.set(CarryFlag, result >= 0);
                cpu.set_zn(result as u8);
            },
            Instruction::CPX => {                   // compare with X register [N,Z,C]
                let result = cpu.x as i16 - operand.get(cpu) as i16;
                cpu.sr.set(CarryFlag, result >= 0);
                cpu.set_zn(result as u8);
            },
            Instruction::CPY => {                   // compare with Y register [N,Z,C]
                let result = cpu.y as i16 - operand.get(cpu) as i16;
                cpu.sr.set(CarryFlag, result >= 0);
                cpu.set_zn(result as u8);
            },
            // Increments & decrements
            Instruction::INC => {                   // increment a memory location [N,Z]
                let value = operand.get(cpu).wrapping_add(1);
                operand.set(cpu, value);
                cpu.set_zn(value);
            },
            Instruction::INX => {                   // increment X register [N,Z]
                let value = cpu.x.wrapping_add(1);
                cpu.x = value;
                cpu.set_zn(value);
            },
            Instruction::INY => {                   // increment Y register [N,Z]
                let value = cpu.y.wrapping_add(1);
                cpu.y = value;
                cpu.set_zn(value);
            },
            Instruction::DEC => {                   // decrement a memory location [N,Z]
                let value = operand.get(cpu).wrapping_sub(1);
                operand.set(cpu, value);
                cpu.set_zn(value);
            },
            Instruction::DEX => {                   // decrement X register [N,Z]
                let value = cpu.x.wrapping_sub(1);
                cpu.x = value;
                cpu.set_zn(value);
            },
            Instruction::DEY => {                   // decrement Y register [N,Z]
                let value = cpu.y.wrapping_sub(1);
                cpu.y = value;
                cpu.set_zn(value);
            },
            // Shifts
            Instruction::ASL => {                   // arithmetic shift left [N,Z,C]
                let value = operand.get(cpu);
                cpu.sr.set(CarryFlag, (value & 0x80) != 0);
                let result = value << 1;
                operand.set(cpu, result);
                cpu.set_zn(result);
            },
            Instruction::LSR => {                   // logical shift right [N,Z,C]
                let value = operand.get(cpu);
                cpu.sr.set(CarryFlag, (value & 0x01) != 0);
                let result = value >> 1;
                operand.set(cpu, result);
                cpu.set_zn(result);
            },
            Instruction::ROL => {                   // rotate left [N,Z,C]
                let carry = cpu.sr.contains(CarryFlag);
                let value = operand.get(cpu);
                cpu.sr.set(CarryFlag, (value & 0x80) != 0);
                let mut result = value << 1;
                if carry { result |= 0x01 }
                operand.set(cpu, result);
                cpu.set_zn(result);
            },
            Instruction::ROR => {                   // rotate right [N,Z,C]
                let carry = cpu.sr.contains(CarryFlag);
                let value = operand.get(cpu);
                cpu.sr.set(CarryFlag, (value & 0x01) != 0);
                let mut result = value >> 1;
                if carry { result |= 0x80 }
                operand.set(cpu, result);
                cpu.set_zn(result);
            },
            // Jump & calls
            Instruction::JMP => {                   // jump to another location
                cpu.pc = operand.addr(cpu);
            },
            Instruction::JSR => {                   // jump to a subroutine
                // Push the address of the last byte of this instruction to the
                // stack instead of the address of the next instruction.
                let pc = cpu.pc; cpu.push(pc - 1);
                cpu.pc = operand.addr(cpu);
            },
            Instruction::RTS => {                   // return from subroutine
                cpu.pc = cpu.pop();
                // Need to advance the PC by 1 to step to the next instruction
                cpu.pc += 1;
            },
            // Branches
            Instruction::BCC => {                   // branch if carry flag clear
                if !cpu.sr.contains(CarryFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            Instruction::BCS => {                   // branch if carry flag set
                if cpu.sr.contains(CarryFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            Instruction::BEQ => {                   // branch if zero flag set
                if cpu.sr.contains(ZeroFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            Instruction::BMI => {                   // branch if negative flag set
                if cpu.sr.contains(NegativeFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            Instruction::BNE => {                   // branch if zero flag clear
                if !cpu.sr.contains(ZeroFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            Instruction::BPL => {                   // branch if negative flag clear
                if !cpu.sr.contains(NegativeFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            Instruction::BVC => {                   // branch if overflow flag clear
                if !cpu.sr.contains(OverflowFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            Instruction::BVS => {                   // branch if overflow flag set
                if cpu.sr.contains(OverflowFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            // Status flag changes
            Instruction::CLC => {                   // clear carry flag [C]
                cpu.sr.remove(CarryFlag);
            },
            Instruction::CLD => {                   // clear decimal mode flag [D]
                cpu.sr.remove(DecimalFlag);
            },
            Instruction::CLI => {                   // clear interrupt disable flag [I]
                cpu.sr.remove(InterruptDisableFlag);
            },
            Instruction::CLV => {                   // clear overflow flag [V]
                cpu.sr.remove(OverflowFlag);
            },
            Instruction::SEC => {                   // set carry flag [C]
                cpu.sr.insert(CarryFlag);
            },
            Instruction::SED => {                   // set decimal mode flag [D]
                cpu.sr.insert(DecimalFlag);
            },
            Instruction::SEI => {                   // set interrupt disable flag [I]
                cpu.sr.insert(InterruptDisableFlag);
            },
            // System functions
            Instruction::BRK => {                   // force an interrupt [B]
                // An IRQ does the same, but clears BreakFlag (before pushing SR).
                cpu.sr.insert(BreakFlag);
                // Unlike JSR, interrupts push the address of the next
                // instruction to the stack. The next byte after BRK is
                // skipped. It can be used to pass information to the
                // interrupt handler.
                let pc = cpu.pc; cpu.push(pc + 1);
                let sr = cpu.sr.bits; cpu.push(sr);
                cpu.sr.insert(InterruptDisableFlag);
                cpu.pc = cpu.mem.get_le(IRQ_VECTOR);
                debug!("mos6502: BRK - Jumping to ({}) -> {}", IRQ_VECTOR.display(), cpu.pc.display());
            },
            Instruction::NOP => {                   // no operation
            },
            Instruction::RTI => {                   // return from interrupt [all]
                cpu.sr.bits = cpu.pop();
                cpu.pc = cpu.pop();
                // Unlike RTS, do not advance the PC since it already points to
                // the next instruction
            },
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Instruction::LDA => "LDA", Instruction::LDX => "LDX", Instruction::LDY => "LDY", Instruction::STA => "STA", Instruction::STX => "STX", Instruction::STY => "STY",
            Instruction::TAX => "TAX", Instruction::TAY => "TAY", Instruction::TXA => "TXA", Instruction::TYA => "TYA",
            Instruction::TSX => "TSX", Instruction::TXS => "TXS", Instruction::PHA => "PHA", Instruction::PHP => "PHP", Instruction::PLA => "PLA", Instruction::PLP => "PLP",
            Instruction::AND => "AND", Instruction::EOR => "EOR", Instruction::ORA => "ORA", Instruction::BIT => "BIT",
            Instruction::ADC => "ADC", Instruction::SBC => "SBC", Instruction::CMP => "CMP", Instruction::CPX => "CPX", Instruction::CPY => "CPY",
            Instruction::INC => "INC", Instruction::INX => "INX", Instruction::INY => "INY", Instruction::DEC => "DEC", Instruction::DEX => "DEX", Instruction::DEY => "DEY",
            Instruction::ASL => "ASL", Instruction::LSR => "LSR", Instruction::ROL => "ROL", Instruction::ROR => "ROR",
            Instruction::JMP => "JMP", Instruction::JSR => "JSR", Instruction::RTS => "RTS",
            Instruction::BCC => "BCC", Instruction::BCS => "BCS", Instruction::BEQ => "BEQ", Instruction::BMI => "BMI", Instruction::BNE => "BNE", Instruction::BPL => "BPL", Instruction::BVC => "BVC", Instruction::BVS => "BVS",
            Instruction::CLC => "CLC", Instruction::CLD => "CLD", Instruction::CLI => "CLI", Instruction::CLV => "CLV", Instruction::SEC => "SEC", Instruction::SED => "SED", Instruction::SEI => "SEI",
            Instruction::BRK => "BRK", Instruction::NOP => "NOP", Instruction::RTI => "RTI",
        })
    }
}
