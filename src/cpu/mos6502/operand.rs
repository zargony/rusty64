//!
//! MOS 6502 operands (adressing modes)
//!

use std::fmt;
use addr::{Address, Masked};
use cpu::Mos6502;
use mem::Addressable;

/// Instruction operand with different addressing modes
#[derive(Debug, PartialEq, Eq)]
pub enum Operand {
    Implied,                            // OPC              Operand implied
    Immediate(u8),                      // OPC #$BB         Operand is value $BB
    Accumulator,                        // OPC A            Operand is AC
    Relative(i8),                       // OPC $RR          Branch target is PC + signed offset $RR (bit 7 signifies negative offset)
    Absolute(u16),                      // OPC $HHLL        Operand is address $HHLL
    AbsoluteIndexedWithX(u16),          // OPC $HHLL,X      Operand is address $HHLL incremented by X
    AbsoluteIndexedWithY(u16),          // OPC $HHLL,Y      Operand is address $HHLL incremented by Y
    Indirect(u16),                      // OPC ($HHLL)      Operand is effective address; effective address is value of address; no page transition (MSB-bug)
    ZeroPage(u8),                       // OPC $LL          Operand is address $00LL
    ZeroPageIndexedWithX(u8),           // OPC $LL,X        Operand is address $00LL incremented by X; no page transition
    ZeroPageIndexedWithY(u8),           // OPC $LL,Y        Operand is address $00LL incremented by Y; no page transition
    ZeroPageIndexedWithXIndirect(u8),   // OPC ($LL,X)      Operand is effective address; effective address is $00LL incremented by X; no page transition
    ZeroPageIndirectIndexedWithY(u8),   // OPC ($LL),Y      Operand is effective address incremented by Y; effective address is word at $00LL
}

impl Operand {
    /// Returns the address an operand targets to
    pub fn addr<M: Addressable> (&self, cpu: &Mos6502<M>) -> u16 {
        match *self {
            Operand::Implied                            => panic!("mos6502: Implied operand does never target an address"),
            Operand::Immediate(..)                      => panic!("mos6502: Immediate operand does never target an address"),
            Operand::Accumulator                        => panic!("mos6502: Accumulator operand does never target an address"),
            Operand::Relative(offset)                   => cpu.pc.offset(offset as i16),
            Operand::Absolute(addr)                     => addr,
            Operand::AbsoluteIndexedWithX(addr)         => addr.offset(cpu.x as i16),
            Operand::AbsoluteIndexedWithY(addr)         => addr.offset(cpu.y as i16),
            Operand::Indirect(addr)                     => cpu.mem.get_le(Masked(addr, 0xff00)),            // simulating MSB-bug
            Operand::ZeroPage(zp)                       => zp as u16,
            Operand::ZeroPageIndexedWithX(zp)           => zp.wrapping_add(cpu.x) as u16,                   // no page transition
            Operand::ZeroPageIndexedWithY(zp)           => zp.wrapping_add(cpu.y) as u16,                   // no page transition
            Operand::ZeroPageIndexedWithXIndirect(zp)   => cpu.mem.get_le(zp.wrapping_add(cpu.x) as u16),   // no page transition
            Operand::ZeroPageIndirectIndexedWithY(zp)   => { let addr: u16 = cpu.mem.get_le(zp as u16); addr.wrapping_add(cpu.y as u16) },
        }
    }

    /// Returns the value an operand specifies
    pub fn get<M: Addressable> (&self, cpu: &Mos6502<M>) -> u8 {
        match *self {
            Operand::Implied                            => panic!("mos6502: Implied operand does never have a value"),
            Operand::Immediate(value)                   => value,
            Operand::Accumulator                        => cpu.ac,
            Operand::Relative(..)                       => panic!("mos6502: Relative operand does never have a value"),
            ref op                                      => cpu.mem.get(op.addr(cpu)),
        }
    }

    /// Sets the value an operand specifies
    pub fn set<M: Addressable> (&self, cpu: &mut Mos6502<M>, value: u8) {
        match *self {
            Operand::Implied                            => panic!("mos6502: Implied operand does never set a value"),
            Operand::Immediate(..)                      => panic!("mos6502: Immediate operand does never set a value"),
            Operand::Accumulator                        => cpu.ac = value,
            Operand::Relative(..)                       => panic!("mos6502: Relative operand does never set a value"),
            ref op                                      => { let addr = op.addr(cpu); cpu.mem.set(addr, value); },
        }
    }
}

impl fmt::Display for Operand {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            Operand::Implied                            => format!(""),
            Operand::Immediate(value)                   => format!("#${:02X}", value),
            Operand::Accumulator                        => format!("A"),
            Operand::Relative(offset)                   => format!("{:+}", offset),
            Operand::Absolute(addr)                     => format!("{}", addr.display()),
            Operand::AbsoluteIndexedWithX(addr)         => format!("{},X", addr.display()),
            Operand::AbsoluteIndexedWithY(addr)         => format!("{},Y", addr.display()),
            Operand::Indirect(addr)                     => format!("({})", addr.display()),
            Operand::ZeroPage(zp)                       => format!("${:02X}", zp),
            Operand::ZeroPageIndexedWithX(zp)           => format!("${:02X},X", zp),
            Operand::ZeroPageIndexedWithY(zp)           => format!("${:02X},Y", zp),
            Operand::ZeroPageIndexedWithXIndirect(zp)   => format!("(${:02X},X)", zp),
            Operand::ZeroPageIndirectIndexedWithY(zp)   => format!("(${:02X}),Y", zp),
        };
        str.fmt(f)
    }
}


#[cfg(test)]
mod tests {
    use cpu::Mos6502;
    use mem::test::TestMemory;
    use super::*;

    #[test]
    fn addressing_modes () {
        let mut cpu = Mos6502::new(TestMemory);
        cpu.pc = 0x1337; cpu.ac = 0x88; cpu.x = 0x11; cpu.y = 0x22;
        // Implied
        let _ = Operand::Implied;
        // Immediate
        assert_eq!(Operand::Immediate(0x55).get(&cpu), 0x55);
        // Accumulator
        assert_eq!(Operand::Accumulator.get(&cpu), 0x88);
        Operand::Accumulator.set(&mut cpu, 0x99);
        assert_eq!(cpu.ac, 0x99);
        // Relative
        assert_eq!(Operand::Relative(0x33).addr(&cpu), 0x136a);
        assert_eq!(Operand::Relative(-0x33).addr(&cpu), 0x1304);
        // Absolute
        assert_eq!(Operand::Absolute(0x0123).addr(&cpu), 0x0123);
        assert_eq!(Operand::Absolute(0x0123).get(&cpu), 0x24);
        Operand::Absolute(0x0123).set(&mut cpu, 0x24);
        // AbsoluteIndexedWithX
        assert_eq!(Operand::AbsoluteIndexedWithX(0x0123).addr(&cpu), 0x0134);
        assert_eq!(Operand::AbsoluteIndexedWithX(0x0123).get(&cpu), 0x35);
        Operand::AbsoluteIndexedWithX(0x0123).set(&mut cpu, 0x35);
        // AbsoluteIndexedWithY
        assert_eq!(Operand::AbsoluteIndexedWithY(0x0123).addr(&cpu), 0x0145);
        assert_eq!(Operand::AbsoluteIndexedWithY(0x0123).get(&cpu), 0x46);
        Operand::AbsoluteIndexedWithY(0x0123).set(&mut cpu, 0x46);
        // Indirect
        assert_eq!(Operand::Indirect(0x0123).addr(&cpu), 0x2524);
        assert_eq!(Operand::Indirect(0x0123).get(&cpu), 0x49);
        Operand::Indirect(0x0123).set(&mut cpu, 0x49);
        // ZeroPage
        assert_eq!(Operand::ZeroPage(0x12).addr(&cpu), 0x0012);
        assert_eq!(Operand::ZeroPage(0x12).get(&cpu), 0x12);
        Operand::ZeroPage(0x12).set(&mut cpu, 0x12);
        // ZeroPageIndexedWithX
        assert_eq!(Operand::ZeroPageIndexedWithX(0x12).addr(&cpu), 0x0023);
        assert_eq!(Operand::ZeroPageIndexedWithX(0x12).get(&cpu), 0x23);
        Operand::ZeroPageIndexedWithX(0x12).set(&mut cpu, 0x23);
        // ZeroPageIndexedWithY
        assert_eq!(Operand::ZeroPageIndexedWithY(0x12).addr(&cpu), 0x0034);
        assert_eq!(Operand::ZeroPageIndexedWithY(0x12).get(&cpu), 0x34);
        Operand::ZeroPageIndexedWithY(0x12).set(&mut cpu, 0x34);
        // ZeroPageIndexedWithXIndirect
        assert_eq!(Operand::ZeroPageIndexedWithXIndirect(0x12).addr(&cpu), 0x2423);
        assert_eq!(Operand::ZeroPageIndexedWithXIndirect(0x12).get(&cpu), 0x47);
        Operand::ZeroPageIndexedWithXIndirect(0x12).set(&mut cpu, 0x47);
        // ZeroPageIndirectIndexedWithY
        assert_eq!(Operand::ZeroPageIndirectIndexedWithY(0x12).addr(&cpu), 0x1334);
        assert_eq!(Operand::ZeroPageIndirectIndexedWithY(0x12).get(&cpu), 0x47);
        Operand::ZeroPageIndirectIndexedWithY(0x12).set(&mut cpu, 0x47);
    }

    #[test]
    fn indirect_addressing_bug () {
        let cpu = Mos6502::new(TestMemory);
        // Indirect($C0FF) must erroneously get address from $C0FF/$C000 instead of $C0FF/$C100
        assert_eq!(Operand::Indirect(0xc0ff).addr(&cpu), 0xc0bf);                   // must be $C0BF, not $C1BF
    }

    #[test]
    fn zero_page_indexed_does_no_page_transition () {
        let mut cpu = Mos6502::new(TestMemory);
        cpu.x = 0x11; cpu.y = 0x22;
        // Zero-page indexed addressing must not transition to the next page
        assert_eq!(Operand::ZeroPageIndexedWithX(0xff).addr(&cpu), 0x0010);         // must be $0010, not $0110
        assert_eq!(Operand::ZeroPageIndexedWithY(0xff).addr(&cpu), 0x0021);         // must be $0021, not $0121
    }

    #[test]
    fn zero_page_indexed_indirect_does_no_page_transition () {
        let mut cpu = Mos6502::new(TestMemory);
        cpu.x = 0x11;
        // Zero-page indexed indirect addressing must not transition to the next page when indexing...
        assert_eq!(Operand::ZeroPageIndexedWithXIndirect(0xff).addr(&cpu), 0x1110); // must be $1110, not $1211
        // ...but may transition to the next page when indirecting
        assert_eq!(Operand::ZeroPageIndexedWithXIndirect(0xee).addr(&cpu), 0x01ff); // must be $01FF, not $00FF
    }

    #[test]
    fn zero_page_indirect_indexed_does_no_page_transition () {
        let mut cpu = Mos6502::new(TestMemory);
        cpu.y = 0x22;
        // Zero-page indirect indexed addressing may transition to the next page when indirecting...
        assert_eq!(Operand::ZeroPageIndirectIndexedWithY(0xff).addr(&cpu), 0x0221); // must be $0221, not $0121
        // ...and may transition to the next page when indexing
        assert_eq!(Operand::ZeroPageIndirectIndexedWithY(0xf0).addr(&cpu), 0xf212); // must be $F212, not $F112
    }
}
