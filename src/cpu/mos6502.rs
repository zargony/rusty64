use std::fmt;
use std::mem::size_of;
use mem::Addressable;
use super::cpu::CPU;

// General information on 65xx: http://en.wikipedia.org/wiki/MOS_Technology_6510
// Useful emulator information: http://emudocs.org/?page=CPU%2065xx
// Web simulator and much info: http://e-tradition.net/bytes/6502/
// Good reference and overview: http://www.obelisk.demon.co.uk/index.html
// Processor bugs and caveats : http://www.textfiles.com/apple/6502.bugs.txt
// Emulator and test resources: http://www.6502.org/tools/emu/

// Test ROMs: http://wiki.nesdev.com/w/index.php/Emulator_tests#CPU
//            http://www.6502.org/tools/emu/
//            http://visual6502.org/wiki/index.php?title=6502TestPrograms
//            http://forum.6502.org/viewtopic.php?f=2&t=2241

/// Processor instructions
#[deriving(Eq)]
enum Instruction {
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
    fn execute<M: Addressable<u16>> (&self, cpu: &mut Mos6502<M>, operand: &Operand) {
        match *self {
            // Load/store operations
            LDA => {                                // load accumulator [N,Z]
                cpu.ac = operand.get(cpu);
                cpu.set_zn(cpu.ac);
            },
            LDX => {                                // load X register [N,Z]
                cpu.x = operand.get(cpu);
                cpu.set_zn(cpu.x);
            },
            LDY => {                                // load Y register [N,Z]
                cpu.y = operand.get(cpu);
                cpu.set_zn(cpu.y);
            },
            STA => {                                // store accumulator
                operand.set(cpu, cpu.ac);
            },
            STX => {                                // store X register
                operand.set(cpu, cpu.x);
            },
            STY => {                                // store Y register
                operand.set(cpu, cpu.y);
            },
            // Register transfers
            TAX => {                                // transfer accumulator to X [N,Z]
                cpu.x = cpu.ac;
                cpu.set_zn(cpu.x);
            },
            TAY => {                                // transfer accumulator to Y [N,Z]
                cpu.y = cpu.ac;
                cpu.set_zn(cpu.y);
            },
            TXA => {                                // transfer X to accumulator [N,Z]
                cpu.ac = cpu.x;
                cpu.set_zn(cpu.ac);
            },
            TYA => {                                // transfer Y to accumulator [N,Z]
                cpu.ac = cpu.y;
                cpu.set_zn(cpu.ac);
            },
            // Stack operations
            TSX => {                                // transfer stack pointer to X [N,Z]
                cpu.x = cpu.sp;
                cpu.set_zn(cpu.x);
            },
            TXS => {                                // transfer X to stack pointer
                cpu.sp = cpu.x;
            },
            PHA => {                                // push accumulator on stack
                cpu.push(cpu.ac);
            },
            PHP => {                                // push processor status (SR) on stack
                cpu.push(cpu.sr);
            },
            PLA => {                                // pull accumulator from stack [N,Z]
                cpu.ac = cpu.pop();
                cpu.set_zn(cpu.ac);
            },
            PLP => {                                // pull processor status (SR) from stack [all]
                cpu.sr = cpu.pop();
                cpu.sr |= 0x20;
            },
            // Logical
            AND => {                                // logical AND [N,Z]
                cpu.ac &= operand.get(cpu);
                cpu.set_zn(cpu.ac);
            },
            EOR => {                                // exclusive OR [N,Z]
                cpu.ac ^= operand.get(cpu);
                cpu.set_zn(cpu.ac);
            },
            ORA => {                                // logical inclusive OR [N,Z]
                cpu.ac |= operand.get(cpu);
                cpu.set_zn(cpu.ac);
            },
            BIT => {                                // bit test [N,V,Z]
                let value = operand.get(cpu);
                cpu.set_flag(ZeroFlag, (value & cpu.ac) == 0);
                cpu.set_flag(NegativeFlag, (value & 0x80) != 0);
                cpu.set_flag(OverflowFlag, (value & 0x40) != 0);
            },
            // Arithmetic
            ADC => {                                // add with carry [N,V,Z,C]
                if cpu.get_flag(DecimalFlag) { fail!("mos6502: Decimal mode ADC not supported yet :("); }
                let value = operand.get(cpu);
                let mut result = cpu.ac as u16 + value as u16;
                if cpu.get_flag(CarryFlag) { result += 1; }
                cpu.set_flag(CarryFlag, (result & 0x100) != 0);
                let result = result as u8;
                cpu.set_flag(OverflowFlag, (cpu.ac ^ value) & 0x80 == 0 && (cpu.ac ^ result) & 0x80 == 0x80);
                cpu.ac = result;
                cpu.set_zn(cpu.ac);
            },
            SBC => {                                // subtract with carry [N,V,Z,C]
                if cpu.get_flag(DecimalFlag) { fail!("mos6502: Decimal mode SBC not supported yet :("); }
                let value = operand.get(cpu);
                let mut result = cpu.ac as u16 - value as u16;
                if !cpu.get_flag(CarryFlag) { result -= 1; }
                cpu.set_flag(CarryFlag, (result & 0x100) == 0);
                let result = result as u8;
                cpu.set_flag(OverflowFlag, (cpu.ac ^ result) & 0x80 != 0 && (cpu.ac ^ value) & 0x80 == 0x80);
                cpu.ac = result;
                cpu.set_zn(cpu.ac);
            },
            CMP => {                                // compare (with accumulator) [N,Z,C]
                let result = cpu.ac as i16 - operand.get(cpu) as i16;
                cpu.set_flag(CarryFlag, result >= 0);
                cpu.set_zn(result as u8);
            },
            CPX => {                                // compare with X register [N,Z,C]
                let result = cpu.x as i16 - operand.get(cpu) as i16;
                cpu.set_flag(CarryFlag, result >= 0);
                cpu.set_zn(result as u8);
            },
            CPY => {                                // compare with Y register [N,Z,C]
                let result = cpu.y as i16 - operand.get(cpu) as i16;
                cpu.set_flag(CarryFlag, result >= 0);
                cpu.set_zn(result as u8);
            },
            // Increments & decrements
            INC => {                                // increment a memory location [N,Z]
                let value = operand.get(cpu) + 1;
                operand.set(cpu, value);
                cpu.set_zn(value);
            },
            INX => {                                // increment X register [N,Z]
                cpu.x += 1;
                cpu.set_zn(cpu.x);
            },
            INY => {                                // increment Y register [N,Z]
                cpu.y += 1;
                cpu.set_zn(cpu.y);
            },
            DEC => {                                // decrement a memory location [N,Z]
                let value = operand.get(cpu) - 1;
                operand.set(cpu, value);
                cpu.set_zn(value);
            },
            DEX => {                                // decrement X register [N,Z]
                cpu.x -= 1;
                cpu.set_zn(cpu.x);
            },
            DEY => {                                // decrement Y register [N,Z]
                cpu.y -= 1;
                cpu.set_zn(cpu.y);
            },
            // Shifts
            ASL => {                                // arithmetic shift left [N,Z,C]
                let value = operand.get(cpu);
                cpu.set_flag(CarryFlag, (value & 0x80) != 0);
                let result = value << 1;
                operand.set(cpu, result);
                cpu.set_zn(result);
            },
            LSR => {                                // logical shift right [N,Z,C]
                let value = operand.get(cpu);
                cpu.set_flag(CarryFlag, (value & 0x01) != 0);
                let result = value >> 1;
                operand.set(cpu, result);
                cpu.set_zn(result);
            },
            ROL => {                                // rotate left [N,Z,C]
                let carry = cpu.get_flag(CarryFlag);
                let value = operand.get(cpu);
                cpu.set_flag(CarryFlag, (value & 0x80) != 0);
                let mut result = value << 1;
                if carry { result |= 0x01 }
                operand.set(cpu, result);
                cpu.set_zn(result);
            },
            ROR => {                                // rotate right [N,Z,C]
                let carry = cpu.get_flag(CarryFlag);
                let value = operand.get(cpu);
                cpu.set_flag(CarryFlag, (value & 0x01) != 0);
                let mut result = value >> 1;
                if carry { result |= 0x80 }
                operand.set(cpu, result);
                cpu.set_zn(result);
            },
            // Jump & calls
            JMP => {                                // jump to another location
                cpu.pc = operand.addr(cpu);
            },
            JSR => {                                // jump to a subroutine
                // Push the address of the last byte of this instruction to the
                // stack instead of the address of the next isntruction.
                cpu.push(cpu.pc - 1);
                cpu.pc = operand.addr(cpu);
            },
            RTS => {                                // return from subroutine
                cpu.pc = cpu.pop();
                // Need to advance the PC by 1 to step to the next instruction
                cpu.pc += 1;
            },
            // Branches
            BCC => {                                // branch if carry flag clear
                if !cpu.get_flag(CarryFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            BCS => {                                // branch if carry flag set
                if cpu.get_flag(CarryFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            BEQ => {                                // branch if zero flag set
                if cpu.get_flag(ZeroFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            BMI => {                                // branch if negative flag set
                if cpu.get_flag(NegativeFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            BNE => {                                // branch if zero flag clear
                if !cpu.get_flag(ZeroFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            BPL => {                                // branch if negative flag clear
                if !cpu.get_flag(NegativeFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            BVC => {                                // branch if overflow flag clear
                if !cpu.get_flag(OverflowFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            BVS => {                                // branch if overflow flag set
                if cpu.get_flag(OverflowFlag) {
                    cpu.pc = operand.addr(cpu);
                }
            },
            // Status flag changes
            CLC => {                                // clear carry flag [C]
                cpu.set_flag(CarryFlag, false);
            },
            CLD => {                                // clear decimal mode flag [D]
                cpu.set_flag(DecimalFlag, false);
            },
            CLI => {                                // clear interrupt disable flag [I]
                cpu.set_flag(InterruptDisableFlag, false);
            },
            CLV => {                                // clear overflow flag [V]
                cpu.set_flag(OverflowFlag, false);
            },
            SEC => {                                // set carry flag [C]
                cpu.set_flag(CarryFlag, true);
            },
            SED => {                                // set decimal mode flag [D]
                cpu.set_flag(DecimalFlag, true);
            },
            SEI => {                                // set interrupt disable flag [I]
                cpu.set_flag(InterruptDisableFlag, true);
            },
            // System functions
            BRK => {                                // force an interrupt [B]
                // An IRQ does the same, but clears BreakFlag (before pushing SR).
                cpu.set_flag(BreakFlag, true);
                // Unlike JSR, interrupts push the address of the next
                // instruction to the stack. The next byte after BRK is
                // skipped. It can be used to pass information to the
                // interrupt handler.
                cpu.push(cpu.pc + 1);
                cpu.push(cpu.sr);
                cpu.set_flag(InterruptDisableFlag, true);
                cpu.pc = cpu.mem.get_le(IRQ_VECTOR);
                debug!("mos6502: BRK - Jumping to (${:04X}) -> ${:04X}", IRQ_VECTOR, cpu.pc);
            },
            NOP => {                                // no operation
            },
            RTI => {                                // return from interrupt [all]
                cpu.sr = cpu.pop();
                cpu.pc = cpu.pop();
                // Unlike RTS, do not advance the PC since it already points to
                // the next instruction
            },
        }
    }
}

impl fmt::Show for Instruction {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(match *self {
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
        })
    }
}

/// Instruction operand with different addressing modes
#[deriving(Eq)]
enum Operand {
    Implied,                            // OPC              operand implied
    Immediate(u8),                      // OPC #$BB         operand is value $BB
    Accumulator,                        // OPC A            operand is AC
    Relative(i8),                       // OPC $RR          branch target is PC + signed offset $RR (bit 7 signifies negative offset)
    Absolute(u16),                      // OPC $HHLL        operand is address $HHLL
    AbsoluteIndexedWithX(u16),          // OPC $HHLL,X      operand is address $HHLL incremented by X
    AbsoluteIndexedWithY(u16),          // OPC $HHLL,Y      operand is address $HHLL incremented by Y
    Indirect(u16),                      // OPC ($HHLL)      operand is effective address; effective address is value of address; no page transition (MSB-bug)
    ZeroPage(u8),                       // OPC $LL          operand is address $00LL
    ZeroPageIndexedWithX(u8),           // OPC $LL,X        operand is address $00LL incremented by X; no page transition
    ZeroPageIndexedWithY(u8),           // OPC $LL,Y        operand is address $00LL incremented by Y; no page transition
    ZeroPageIndexedWithXIndirect(u8),   // OPC ($LL,X)      operand is effective address; effective address is $00LL incremented by X; no page transition
    ZeroPageIndirectIndexedWithY(u8),   // OPC ($LL),Y      operand is effective address incremented by Y; effective address is word at $00LL
}

impl Operand {
    /// Returns the address an operand targets to
    fn addr<M: Addressable<u16>> (&self, cpu: &Mos6502<M>) -> u16 {
        match *self {
            Implied                             => fail!("mos6502: Implied operand does never target an address"),
            Immediate(..)                       => fail!("mos6502: Immediate operand does never target an address"),
            Accumulator                         => fail!("mos6502: Accumulator operand does never target an address"),
            Relative(offset)                    => cpu.pc + offset as u16,
            Absolute(addr)                      => addr,
            AbsoluteIndexedWithX(addr)          => addr + cpu.x as u16,
            AbsoluteIndexedWithY(addr)          => addr + cpu.y as u16,
            Indirect(addr)                      => cpu.mem.get_le_masked(addr, 0x00ff),         // simulating MSB-bug
            ZeroPage(addr)                      => addr as u16,
            ZeroPageIndexedWithX(addr)          => (addr + cpu.x) as u16,                       // no page transition
            ZeroPageIndexedWithY(addr)          => (addr + cpu.y) as u16,                       // no page transition
            ZeroPageIndexedWithXIndirect(addr)  => cpu.mem.get_le((addr + cpu.x) as u16),       // no page transition
            ZeroPageIndirectIndexedWithY(addr)  => cpu.mem.get_le::<u16>(addr as u16) + cpu.y as u16,
        }
    }

    /// Returns the value an operand specifies
    fn get<M: Addressable<u16>> (&self, cpu: &Mos6502<M>) -> u8 {
        match *self {
            Implied                             => fail!("mos6502: Implied operand does never have a value"),
            Immediate(value)                    => value,
            Accumulator                         => cpu.ac,
            Relative(..)                        => fail!("mos6502: Relative operand does never have a value"),
            op                                  => cpu.mem.get(op.addr(cpu)),
        }
    }

    /// Sets the value an operand specifies
    fn set<M: Addressable<u16>> (&self, cpu: &mut Mos6502<M>, value: u8) {
        match *self {
            Implied                             => fail!("mos6502: Implied operand does never set a value"),
            Immediate(..)                       => fail!("mos6502: Immediate operand does never set a value"),
            Accumulator                         => cpu.ac = value,
            Relative(..)                        => fail!("mos6502: Relative operand does never set a value"),
            op                                  => { let addr = op.addr(cpu); cpu.mem.set(addr, value); },
        }
    }
}

impl fmt::Show for Operand {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(match *self {
            Implied                             => ~"",
            Immediate(value)                    => format!("\\#${:02X}", value),
            Accumulator                         => ~"A",
            Relative(offset)                    => format!("{:+d}", offset),
            Absolute(addr)                      => format!("${:04X}", addr),
            AbsoluteIndexedWithX(addr)          => format!("${:04X},X", addr),
            AbsoluteIndexedWithY(addr)          => format!("${:04X},Y", addr),
            Indirect(addr)                      => format!("(${:04X})", addr),
            ZeroPage(addr)                      => format!("${:02X}", addr),
            ZeroPageIndexedWithX(addr)          => format!("${:02X},X", addr),
            ZeroPageIndexedWithY(addr)          => format!("${:02X},Y", addr),
            ZeroPageIndexedWithXIndirect(addr)  => format!("(${:02X},X)", addr),
            ZeroPageIndirectIndexedWithY(addr)  => format!("(${:02X}),Y", addr),
        })
    }
}

/// Hard-coded address where to look for the address to jump to on nonmaskable interrupt
static NMI_VECTOR:          u16 = 0xfffa;
/// Hard-coded address where to look for the address to jump to on reset
static RESET_VECTOR:        u16 = 0xfffc;
/// Hard-coded address where to look for the address to jump to on interrupt
static IRQ_VECTOR:          u16 = 0xfffe;

/// The MOS6502 processor
pub struct Mos6502<M> {
    pc: u16,                            // Program Counter
    ac: u8,                             // Accumulator
    x: u8,                              // X register
    y: u8,                              // Y register
    sr: u8,                             // Status Register (NV-BDIZC: Negative, oVerflow, 1, Break, Decimal, Interrupt, Zero, Carry)
    sp: u8,                             // Stack Pointer
    mem: M,                             // main memory
    reset: bool,                        // RESET line
    nmi: bool,                          // NMI line
    irq: bool,                          // IRQ line
}

/// The MOS6502 status flags
pub enum StatusFlag {
    CarryFlag               = 0,
    ZeroFlag                = 1,
    InterruptDisableFlag    = 2,
    DecimalFlag             = 3,
    BreakFlag               = 4,
    OverflowFlag            = 6,
    NegativeFlag            = 7,
}

impl<M: Addressable<u16>> Mos6502<M> {
    /// Create a new MOS6502 processor
    pub fn new (mem: M) -> Mos6502<M> {
        Mos6502 { pc: 0, ac: 0, x: 0, y: 0, sr: 0x20, sp: 0, mem: mem, reset: true, nmi: false, irq: false }
    }

    /// Get the memory contents at the current PC and advance the PC
    fn next<T: Primitive> (&mut self) -> T {
        let val = self.mem.get_le(self.pc);
        self.pc += size_of::<T>() as u16;
        val
    }

    /// Parse next instruction and advance PC. Returns number of cycles, instruction and operand
    fn next_instruction (&mut self) -> Option<(uint, Instruction, Operand)> {
        let opcode: u8 = self.next();
        Some(match opcode {
            // Load/store operations
            0xa9 => (2, LDA, Immediate(self.next())),
            0xa5 => (3, LDA, ZeroPage(self.next())),
            0xb5 => (4, LDA, ZeroPageIndexedWithX(self.next())),
            0xad => (4, LDA, Absolute(self.next())),
            0xbd => (4, LDA, AbsoluteIndexedWithX(self.next())),                // +1 cycle if page crossed
            0xb9 => (4, LDA, AbsoluteIndexedWithY(self.next())),                // +1 cycle if page crossed
            0xa1 => (6, LDA, ZeroPageIndexedWithXIndirect(self.next())),
            0xb1 => (5, LDA, ZeroPageIndirectIndexedWithY(self.next())),        // +1 cycle if page crossed
            0xa2 => (2, LDX, Immediate(self.next())),
            0xa6 => (3, LDX, ZeroPage(self.next())),
            0xb6 => (4, LDX, ZeroPageIndexedWithY(self.next())),
            0xae => (4, LDX, Absolute(self.next())),
            0xbe => (4, LDX, AbsoluteIndexedWithY(self.next())),                // +1 cycle if page crossed
            0xa0 => (2, LDY, Immediate(self.next())),
            0xa4 => (3, LDY, ZeroPage(self.next())),
            0xb4 => (4, LDY, ZeroPageIndexedWithX(self.next())),
            0xac => (4, LDY, Absolute(self.next())),
            0xbc => (4, LDY, AbsoluteIndexedWithX(self.next())),                // +1 cycle if page crossed
            0x85 => (3, STA, ZeroPage(self.next())),
            0x95 => (4, STA, ZeroPageIndexedWithX(self.next())),
            0x8d => (4, STA, Absolute(self.next())),
            0x9d => (5, STA, AbsoluteIndexedWithX(self.next())),
            0x99 => (5, STA, AbsoluteIndexedWithY(self.next())),
            0x81 => (6, STA, ZeroPageIndexedWithXIndirect(self.next())),
            0x91 => (6, STA, ZeroPageIndirectIndexedWithY(self.next())),
            0x86 => (3, STX, ZeroPage(self.next())),
            0x96 => (4, STX, ZeroPageIndexedWithY(self.next())),
            0x8e => (4, STX, Absolute(self.next())),
            0x84 => (3, STY, ZeroPage(self.next())),
            0x94 => (4, STY, ZeroPageIndexedWithX(self.next())),
            0x8c => (4, STY, Absolute(self.next())),
            // Register transfers
            0xaa => (2, TAX, Implied),
            0xa8 => (2, TAY, Implied),
            0x8a => (2, TXA, Implied),
            0x98 => (2, TYA, Implied),
            // Stack operations
            0xba => (2, TSX, Implied),
            0x9a => (2, TXS, Implied),
            0x48 => (3, PHA, Implied),
            0x08 => (3, PHP, Implied),
            0x68 => (4, PLA, Implied),
            0x28 => (4, PLP, Implied),
            // Logical
            0x29 => (2, AND, Immediate(self.next())),
            0x25 => (3, AND, ZeroPage(self.next())),
            0x35 => (4, AND, ZeroPageIndexedWithX(self.next())),
            0x2d => (4, AND, Absolute(self.next())),
            0x3d => (4, AND, AbsoluteIndexedWithX(self.next())),                // +1 cycle if page crossed
            0x39 => (4, AND, AbsoluteIndexedWithY(self.next())),                // +1 cycle if page crossed
            0x21 => (6, AND, ZeroPageIndexedWithXIndirect(self.next())),
            0x31 => (5, AND, ZeroPageIndirectIndexedWithY(self.next())),        // +1 cycle if page crossed
            0x49 => (2, EOR, Immediate(self.next())),
            0x45 => (3, EOR, ZeroPage(self.next())),
            0x55 => (4, EOR, ZeroPageIndexedWithX(self.next())),
            0x4d => (4, EOR, Absolute(self.next())),
            0x5d => (4, EOR, AbsoluteIndexedWithX(self.next())),                // +1 cycle if page crossed
            0x59 => (4, EOR, AbsoluteIndexedWithY(self.next())),                // +1 cycle if page crossed
            0x41 => (6, EOR, ZeroPageIndexedWithXIndirect(self.next())),
            0x51 => (5, EOR, ZeroPageIndirectIndexedWithY(self.next())),        // +1 cycle if page crossed
            0x09 => (2, ORA, Immediate(self.next())),
            0x05 => (3, ORA, ZeroPage(self.next())),
            0x15 => (4, ORA, ZeroPageIndexedWithX(self.next())),
            0x0d => (4, ORA, Absolute(self.next())),
            0x1d => (4, ORA, AbsoluteIndexedWithX(self.next())),                // +1 cycle if page crossed
            0x19 => (4, ORA, AbsoluteIndexedWithY(self.next())),                // +1 cycle if page crossed
            0x01 => (6, ORA, ZeroPageIndexedWithXIndirect(self.next())),
            0x11 => (5, ORA, ZeroPageIndirectIndexedWithY(self.next())),        // +1 cycle if page crossed
            0x24 => (3, BIT, ZeroPage(self.next())),
            0x2c => (4, BIT, Absolute(self.next())),
            // Arithmetic
            0x69 => (2, ADC, Immediate(self.next())),
            0x65 => (3, ADC, ZeroPage(self.next())),
            0x75 => (4, ADC, ZeroPageIndexedWithX(self.next())),
            0x6d => (4, ADC, Absolute(self.next())),
            0x7d => (4, ADC, AbsoluteIndexedWithX(self.next())),                // +1 cycle if page crossed
            0x79 => (4, ADC, AbsoluteIndexedWithY(self.next())),                // +1 cycle if page crossed
            0x61 => (6, ADC, ZeroPageIndexedWithXIndirect(self.next())),
            0x71 => (5, ADC, ZeroPageIndirectIndexedWithY(self.next())),        // +1 cycle if page crossed
            0xe9 => (2, SBC, Immediate(self.next())),
            0xe5 => (3, SBC, ZeroPage(self.next())),
            0xf5 => (4, SBC, ZeroPageIndexedWithX(self.next())),
            0xed => (4, SBC, Absolute(self.next())),
            0xfd => (4, SBC, AbsoluteIndexedWithX(self.next())),                // +1 cycle if page crossed
            0xf9 => (4, SBC, AbsoluteIndexedWithY(self.next())),                // +1 cycle if page crossed
            0xe1 => (6, SBC, ZeroPageIndexedWithXIndirect(self.next())),
            0xf1 => (5, SBC, ZeroPageIndirectIndexedWithY(self.next())),        // +1 cycle if page crossed
            0xc9 => (2, CMP, Immediate(self.next())),
            0xc5 => (3, CMP, ZeroPage(self.next())),
            0xd5 => (4, CMP, ZeroPageIndexedWithX(self.next())),
            0xcd => (4, CMP, Absolute(self.next())),
            0xdd => (4, CMP, AbsoluteIndexedWithX(self.next())),                // +1 cycle if page crossed
            0xd9 => (4, CMP, AbsoluteIndexedWithY(self.next())),                // +1 cycle if page crossed
            0xc1 => (6, CMP, ZeroPageIndexedWithXIndirect(self.next())),
            0xd1 => (5, CMP, ZeroPageIndirectIndexedWithY(self.next())),        // +1 cycle if page crossed
            0xe0 => (2, CPX, Immediate(self.next())),
            0xe4 => (3, CPX, ZeroPage(self.next())),
            0xec => (4, CPX, Absolute(self.next())),
            0xc0 => (2, CPY, Immediate(self.next())),
            0xc4 => (3, CPY, ZeroPage(self.next())),
            0xcc => (4, CPY, Absolute(self.next())),
            // Increments & decrements
            0xe6 => (5, INC, ZeroPage(self.next())),
            0xf6 => (6, INC, ZeroPageIndexedWithX(self.next())),
            0xee => (6, INC, Absolute(self.next())),
            0xfe => (7, INC, AbsoluteIndexedWithX(self.next())),
            0xe8 => (2, INX, Implied),
            0xc8 => (2, INY, Implied),
            0xc6 => (5, DEC, ZeroPage(self.next())),
            0xd6 => (6, DEC, ZeroPageIndexedWithX(self.next())),
            0xce => (6, DEC, Absolute(self.next())),
            0xde => (7, DEC, AbsoluteIndexedWithX(self.next())),
            0xca => (2, DEX, Implied),
            0x88 => (2, DEY, Implied),
            // Shifts
            0x0a => (2, ASL, Accumulator),
            0x06 => (5, ASL, ZeroPage(self.next())),
            0x16 => (6, ASL, ZeroPageIndexedWithX(self.next())),
            0x0e => (6, ASL, Absolute(self.next())),
            0x1e => (7, ASL, AbsoluteIndexedWithX(self.next())),
            0x4a => (2, LSR, Accumulator),
            0x46 => (5, LSR, ZeroPage(self.next())),
            0x56 => (6, LSR, ZeroPageIndexedWithX(self.next())),
            0x4e => (6, LSR, Absolute(self.next())),
            0x5e => (7, LSR, AbsoluteIndexedWithX(self.next())),
            0x2a => (2, ROL, Accumulator),
            0x26 => (5, ROL, ZeroPage(self.next())),
            0x36 => (6, ROL, ZeroPageIndexedWithX(self.next())),
            0x2e => (6, ROL, Absolute(self.next())),
            0x3e => (7, ROL, AbsoluteIndexedWithX(self.next())),
            0x6a => (2, ROR, Accumulator),
            0x66 => (5, ROR, ZeroPage(self.next())),
            0x76 => (6, ROR, ZeroPageIndexedWithX(self.next())),
            0x6e => (6, ROR, Absolute(self.next())),
            0x7e => (7, ROR, AbsoluteIndexedWithX(self.next())),
            // Jump & calls
            0x4c => (3, JMP, Absolute(self.next())),
            0x6c => (5, JMP, Indirect(self.next())),
            0x20 => (6, JSR, Absolute(self.next())),
            0x60 => (6, RTS, Implied),
            // Branches
            0x90 => (2, BCC, Relative(self.next())),                            // +1 cycle if branched, +2 if page crossed
            0xb0 => (2, BCS, Relative(self.next())),                            // +1 cycle if branched, +2 if page crossed
            0xf0 => (2, BEQ, Relative(self.next())),                            // +1 cycle if branched, +2 if page crossed
            0x30 => (2, BMI, Relative(self.next())),                            // +1 cycle if branched, +2 if page crossed
            0xd0 => (2, BNE, Relative(self.next())),                            // +1 cycle if branched, +2 if page crossed
            0x10 => (2, BPL, Relative(self.next())),                            // +1 cycle if branched, +2 if page crossed
            0x50 => (2, BVC, Relative(self.next())),                            // +1 cycle if branched, +2 if page crossed
            0x70 => (2, BVS, Relative(self.next())),                            // +1 cycle if branched, +2 if page crossed
            // Status flag changes
            0x18 => (2, CLC, Implied),
            0xd8 => (2, CLD, Implied),
            0x58 => (2, CLI, Implied),
            0xb8 => (2, CLV, Implied),
            0x38 => (2, SEC, Implied),
            0xf8 => (2, SED, Implied),
            0x78 => (2, SEI, Implied),
            // System functions
            0x00 => (7, BRK, Implied),
            0xea => (2, NOP, Implied),
            0x40 => (6, RTI, Implied),
            // Illegal opcode
            _ => return None,
        })
    }

    /// Get the given status flag
    fn get_flag (&self, flag: StatusFlag) -> bool {
        (self.sr & (1 << flag as u8)) != 0
    }

    /// Set the given status flag to the given state
    fn set_flag (&mut self, flag: StatusFlag, state: bool) {
        if state {
            self.sr |= 1 << flag as u8;
        } else {
            self.sr &= !(1 << flag as u8);
        }
    }

    /// Set ZeroFlag and NegativeFlag based on the given value
    fn set_zn (&mut self, value: u8) -> u8 {
        self.set_flag(ZeroFlag, value == 0);
        self.set_flag(NegativeFlag, (value as i8) < 0);
        value
    }

    /// Push a value onto the stack
    fn push<T: Primitive> (&mut self, value: T) {
        // SP points to the next free stack position as $0100+SP. SP needs to be
        // initialized to #$FF by the reset code. As the stack grows, SP decreases
        // down to #$00 (i.e. stack full). Stack access never leaves the stack page!
        self.sp -= size_of::<T>() as u8;
        self.mem.set_le_masked(0x0100 + (self.sp + 1) as u16, 0x00ff, value);
    }

    /// Pop a value from the stack
    fn pop<T: Primitive> (&mut self) -> T {
        // See push() for details
        let value: T = self.mem.get_le_masked(0x0100 + (self.sp + 1) as u16, 0x00ff);
        self.sp += size_of::<T>() as u8;
        value
    }

    /// Interrupt the CPU (NMI)
    #[allow(dead_code)]
    pub fn nmi (&mut self) {
        // Trigger the NMI line. The actual NMI processing is done in the next step().
        self.nmi = true;
    }

    /// Interrupt the CPU (IRQ)
    #[allow(dead_code)]
    pub fn irq (&mut self) {
        // Trigger the IRQ line. The actual IRQ processing is done in the next step().
        self.irq = true;
    }
}

impl<M: Addressable<u16>> CPU for Mos6502<M> {
    /// Reset the CPU
    fn reset (&mut self) {
        // Trigger the RESET line. The actual RESET processing is done in the next step().
        self.reset = true;
    }

    /// Do one step (execute the next instruction). Returns the number of
    /// cycles the instruction needed
    fn step (&mut self) -> uint {
        // Process RESET if line was triggered
        if self.reset {
            // A RESET jumps to the vector at RESET_VECTOR and sets the InterruptDisableFlag.
            // Note that all other states and registers are unspecified and might contain
            // random values, so they need to be initialized by the reset routine.
            // See also http://6502.org/tutorials/interrupts.html
            self.set_flag(InterruptDisableFlag, true);
            self.pc = self.mem.get_le(RESET_VECTOR);
            self.reset = false;
            self.nmi = false;
            self.irq = false;
            debug!("mos6502: RESET - Jumping to (${:04X}) -> ${:04X}", RESET_VECTOR, self.pc);
            return 6;
        }
        // Process NMI if line was triggered
        if self.nmi {
            // An NMI pushes PC and SR to the stack and jumps to the vector at NMI_VECTOR.
            // It does NOT set the InterruptDisableFlag. Unlike JSR, it pushes the address
            // of the next instruction to the stack.
            // See also http://6502.org/tutorials/interrupts.html
            self.push(self.pc);
            self.push(self.sr);
            self.pc = self.mem.get_le(NMI_VECTOR);
            self.nmi = false;
            debug!("mos6502: NMI - Jumping to (${:04X}) -> ${:04X}", NMI_VECTOR, self.pc);
            return 7;
        }
        // Process IRQ if line was triggered and interrupts are enabled
        if self.irq && !self.get_flag(InterruptDisableFlag) {
            // An IRQ pushes PC and SR to the stack, jumps to the vector at IRQ_VECTOR and
            // sets the InterruptDisableFlag. Unlike JSR, it pushes the address of the next
            // instruction to the stack. This also emulates the BRK bug where a BRK instruction
            // is ignored if an IRQ occurs simultaneously.
            // The BRK instruction does the same, but sets BreakFlag (before pushing SR).
            // See also http://6502.org/tutorials/interrupts.html
            self.set_flag(BreakFlag, false);
            if self.mem.get(self.pc) == 0x00 { self.pc += 1; }    // Emulate BRK bug
            self.push(self.pc);
            self.push(self.sr);
            self.set_flag(InterruptDisableFlag, true);
            self.pc = self.mem.get_le(IRQ_VECTOR);
            // FIXME: The real 6502 IRQ line is level-sensitive, not edge-sensitive!
            // FIXME: I.e. it does not stop jumping to the IRQ_VECTOR after one run,
            // FIXME: but after the hardware drops the IRQ line (which the interrupt
            // FIXME: code usually causes, but not necessary needs to cause).
            self.irq = false;
            debug!("mos6502: IRQ - Jumping to (${:04X}) -> ${:04X}", IRQ_VECTOR, self.pc);
            return 7;
        }
        // Read and parse next opcode
        let old_pc = self.pc;
        match self.next_instruction() {
            // Got valid opcode
            Some((cycles, instruction, operand)) => {
                let new_pc = self.pc;
                instruction.execute(self, &operand);
                debug!("mos6502: {:04X}  {:-8s}  {:-3} {:-26}  -[{:u}]-> AC:{:02X} X:{:02X} Y:{:02X} SR:{:02X} SP:{:02X} NV-BDIZC:{:08t}",
                    old_pc, self.mem.hexdump(old_pc, new_pc), instruction, operand,
                    cycles, self.ac, self.x, self.y, self.sr, self.sp, self.sr);
                cycles
            },
            // Got illegal opcode
            None => {
                debug!("mos6502: {:04X}  {:-8s}  ???", old_pc, self.mem.hexdump(old_pc, old_pc+3));
                fail!("mos6502: Illegal opcode ${:02X} at ${:04X}", self.mem.get(old_pc), old_pc)
            },
        }
    }
}


#[cfg(test)]
mod test {
    use mem::{Addressable, Ram, Rom};
    use super::super::CPU;
    use super::LDA;
    use super::{Immediate, Accumulator, Relative, Absolute, AbsoluteIndexedWithX, AbsoluteIndexedWithY, Indirect};
    use super::{ZeroPage, ZeroPageIndexedWithX, ZeroPageIndexedWithY, ZeroPageIndexedWithXIndirect, ZeroPageIndirectIndexedWithY};
    use super::{CarryFlag, ZeroFlag, BreakFlag, OverflowFlag, NegativeFlag};
    use super::Mos6502;

    /// Test-memory that returns/expects the lower nibble of the address as data
    struct TestMemory;
    impl Addressable<u16> for TestMemory {
        fn get (&self, addr: u16) -> u8 { addr as u8 }
        fn set (&mut self, addr: u16, data: u8) { assert_eq!(data, addr as u8); }
    }

    /// Test-memory that returns/expects the sum of the lower and higher nibble of the address as data
    struct SpecialTestMemory;
    impl Addressable<u16> for SpecialTestMemory {
        fn get (&self, addr: u16) -> u8 { addr as u8 + (addr >> 8) as u8 }
        fn set (&mut self, addr: u16, data: u8) { assert_eq!(data, addr as u8 + (addr >> 8) as u8); }
    }

    /// Create a cpu using the given memory for testing purpose
    fn test_cpu<M: Addressable<u16>> (mem: M) -> Mos6502<M> {
        let mut cpu = Mos6502::new(mem);
        cpu.reset = false;
        assert!(!cpu.reset && !cpu.nmi && !cpu.irq);
        cpu
    }

    #[test]
    fn smoke () {
        let mut cpu = test_cpu(TestMemory);
        cpu.reset();
        cpu.nmi();
        cpu.irq();
        cpu.step();
    }

    #[test]
    fn addressing_modes () {
        let mut cpu = test_cpu(TestMemory);
        cpu.pc = 0x1337; cpu.ac = 0x88; cpu.x = 0x11; cpu.y = 0x22;
        // Immediate
        assert_eq!(Immediate(0x55).get(&cpu), 0x55);
        // Accumulator
        assert_eq!(Accumulator.get(&cpu), 0x88);
        Accumulator.set(&mut cpu, 0x99); assert_eq!(cpu.ac, 0x99);
        // Relative
        assert_eq!(Relative(0x33).addr(&cpu), 0x136a);
        assert_eq!(Relative(-0x33).addr(&cpu), 0x1304);
        // Absolute
        assert_eq!(Absolute(0x0123).addr(&cpu), 0x0123);
        assert_eq!(Absolute(0x0123).get(&cpu), 0x23);
        Absolute(0x0123).set(&mut cpu, 0x23);
        // AbsoluteIndexedWithX
        assert_eq!(AbsoluteIndexedWithX(0x0123).addr(&cpu), 0x0134);
        assert_eq!(AbsoluteIndexedWithX(0x0123).get(&cpu), 0x34);
        AbsoluteIndexedWithX(0x0123).set(&mut cpu, 0x34);
        // AbsoluteIndexedWithY
        assert_eq!(AbsoluteIndexedWithY(0x0123).addr(&cpu), 0x0145);
        assert_eq!(AbsoluteIndexedWithY(0x0123).get(&cpu), 0x45);
        AbsoluteIndexedWithY(0x0123).set(&mut cpu, 0x45);
        // Indirect
        assert_eq!(Indirect(0x0123).addr(&cpu), 0x2423);
        assert_eq!(Indirect(0x0123).get(&cpu), 0x23);
        Indirect(0x0123).set(&mut cpu, 0x23);
        // ZeroPage
        assert_eq!(ZeroPage(0x12).addr(&cpu), 0x0012);
        assert_eq!(ZeroPage(0x12).get(&cpu), 0x12);
        ZeroPage(0x12).set(&mut cpu, 0x12);
        // ZeroPageIndexedWithX
        assert_eq!(ZeroPageIndexedWithX(0x12).addr(&cpu), 0x0023);
        assert_eq!(ZeroPageIndexedWithX(0x12).get(&cpu), 0x23);
        ZeroPageIndexedWithX(0x12).set(&mut cpu, 0x23);
        // ZeroPageIndexedWithY
        assert_eq!(ZeroPageIndexedWithY(0x12).addr(&cpu), 0x0034);
        assert_eq!(ZeroPageIndexedWithY(0x12).get(&cpu), 0x34);
        ZeroPageIndexedWithY(0x12).set(&mut cpu, 0x34);
        // ZeroPageIndexedWithXIndirect
        assert_eq!(ZeroPageIndexedWithXIndirect(0x12).addr(&cpu), 0x2423);
        assert_eq!(ZeroPageIndexedWithXIndirect(0x12).get(&cpu), 0x23);
        ZeroPageIndexedWithXIndirect(0x12).set(&mut cpu, 0x23);
        // ZeroPageIndirectIndexedWithY
        assert_eq!(ZeroPageIndirectIndexedWithY(0x12).addr(&cpu), 0x1334);
        assert_eq!(ZeroPageIndirectIndexedWithY(0x12).get(&cpu), 0x34);
        ZeroPageIndirectIndexedWithY(0x12).set(&mut cpu, 0x34);
    }

    #[test]
    fn indirect_addressing_bug () {
        let cpu = test_cpu(SpecialTestMemory);
        // Indirect($c0ff) must erroneously get address from $c0ff/$c000 instead of $c0ff/$c100
        assert_eq!(Indirect(0xc0ff).addr(&cpu), 0xc0bf);                    // must be $c0bf, not $c1bf
    }

    #[test]
    fn zero_page_indexed_page_transition () {
        let mut cpu = test_cpu(SpecialTestMemory);
        cpu.x = 0x11; cpu.y = 0x22;
        // Zero-page indexed addressing must not transition to the next page
        assert_eq!(ZeroPageIndexedWithX(0xff).addr(&cpu), 0x0010);          // must be $0010, not $0110
        assert_eq!(ZeroPageIndexedWithY(0xff).addr(&cpu), 0x0021);          // must be $0021, not $0121
    }

    #[test]
    fn zero_page_indexed_indirect_page_transition () {
        let mut cpu = test_cpu(SpecialTestMemory);
        cpu.x = 0x11;
        // Zero-page indexed indirect addressing must not transition to the next page when indexing...
        assert_eq!(ZeroPageIndexedWithXIndirect(0xff).addr(&cpu), 0x1110);  // must be $1110, not $1211
        // ...but may transition to the next page when indirecting
        assert_eq!(ZeroPageIndexedWithXIndirect(0xee).addr(&cpu), 0x01ff);  // must be $01ff, not $00ff
    }

    #[test]
    fn zero_page_indirect_indexed_page_transition () {
        let mut cpu = test_cpu(SpecialTestMemory);
        cpu.y = 0x22;
        // Zero-page indirect indexed addressing may transition to the next page when indirecting...
        assert_eq!(ZeroPageIndirectIndexedWithY(0xff).addr(&cpu), 0x0221);  // must be $0221, not $0121
        // ...and may transition to the next page when indexing
        assert_eq!(ZeroPageIndirectIndexedWithY(0xf0).addr(&cpu), 0xf212);  // must be $f212, not $f112
    }

    #[test]
    fn initial_state () {
        let cpu = Mos6502::new(TestMemory);
        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sr, 0x20);
        assert!(cpu.reset);
    }

    #[test]
    fn fetch_memory_contents_and_advance_pc () {
        let mut cpu = test_cpu(TestMemory);
        cpu.pc = 0x0012;
        let val: u8 = cpu.next(); assert_eq!(val, 0x12);
        let val: u8 = cpu.next(); assert_eq!(val, 0x13);
        let val: u16 = cpu.next(); assert_eq!(val, 0x1514);
        let val: u16 = cpu.next(); assert_eq!(val, 0x1716);
    }

    #[test]
    fn fetch_instruction_and_advance_pc () {
        let mut cpu = test_cpu(TestMemory);
        cpu.pc = 0x00ad;                    // AD AE AF: LDA $AFAE
        assert_eq!(cpu.next_instruction(), Some((4, LDA, Absolute(0xafae))));
    }

    #[test]
    fn status_flags () {
        let mut cpu = test_cpu(TestMemory);
        cpu.sr = 0xaa;
        assert!(!cpu.get_flag(CarryFlag));
        assert!(cpu.get_flag(ZeroFlag));
        assert!(!cpu.get_flag(OverflowFlag));
        assert!(cpu.get_flag(NegativeFlag));
        cpu.set_flag(CarryFlag, true);
        cpu.set_flag(ZeroFlag, false);
        cpu.set_flag(OverflowFlag, true);
        cpu.set_flag(NegativeFlag, false);
        assert_eq!(cpu.sr, 0x69);
    }

    #[test]
    fn zero_and_negative_values () {
        let mut cpu = test_cpu(TestMemory);
        cpu.set_zn(0);
        assert!(cpu.get_flag(ZeroFlag));
        assert!(!cpu.get_flag(NegativeFlag));
        cpu.set_zn(42);
        assert!(!cpu.get_flag(ZeroFlag));
        assert!(!cpu.get_flag(NegativeFlag));
        cpu.set_zn(142);
        assert!(!cpu.get_flag(ZeroFlag));
        assert!(cpu.get_flag(NegativeFlag));
    }

    #[test]
    fn stack_push_pop () {
        let mut cpu = test_cpu(Ram::with_capacity(0x01ff_u16));
        cpu.sp = 0xff;
        assert_eq!(cpu.mem.get(0x01ff), 0x00);
        cpu.push(0x12_u8);
        assert_eq!(cpu.sp, 0xfe);
        assert_eq!(cpu.mem.get(0x01ff), 0x12);
        assert_eq!(cpu.mem.get(0x01fe), 0x00);
        cpu.push(0x3456_u16);
        assert_eq!(cpu.sp, 0xfc);
        assert_eq!(cpu.mem.get(0x01fe), 0x34);
        assert_eq!(cpu.mem.get(0x01fd), 0x56);
        assert_eq!(cpu.mem.get(0x01fc), 0x00);
        let val: u8 = cpu.pop();
        assert_eq!(val, 0x56);
        assert_eq!(cpu.sp, 0xfd);
        let val: u16 = cpu.pop();
        assert_eq!(val, 0x1234);
        assert_eq!(cpu.sp, 0xff);
    }

    #[test]
    fn stack_overflow () {
        let mut cpu = test_cpu(Ram::with_capacity(0x01ff_u16));
        cpu.sp = 0x00;
        cpu.push(0x12_u8);
        assert_eq!(cpu.sp, 0xff);
        assert_eq!(cpu.mem.get(0x0100), 0x12);
        let val: u8 = cpu.pop();
        assert_eq!(val, 0x12);
        assert_eq!(cpu.sp, 0x00);
    }

    #[test]
    fn stack_overflow_word () {
        let mut cpu = test_cpu(Ram::with_capacity(0x01ff_u16));
        cpu.sp = 0x00;
        cpu.push(0x1234_u16);
        assert_eq!(cpu.sp, 0xfe);
        assert_eq!(cpu.mem.get(0x0100), 0x12);
        assert_eq!(cpu.mem.get(0x01ff), 0x34);
        let val: u16 = cpu.pop();
        assert_eq!(val, 0x1234);
        assert_eq!(cpu.sp, 0x00);
    }

    #[test]
    fn state_after_reset () {
        let mut cpu = test_cpu(Ram::with_capacity(0xffff_u16));
        cpu.sr = 0x23;
        cpu.sp = 0xff;
        cpu.mem.set_le(0xfffc, 0x1234);
        cpu.reset();
        cpu.step();
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sr, 0x27);
        assert_eq!(cpu.sp, 0xff);
    }

    #[test]
    fn state_after_nmi () {
        let mut cpu = test_cpu(Ram::with_capacity(0xffff_u16));
        cpu.sr = 0x23;
        cpu.sp = 0xff;
        cpu.mem.set_le(0xfffa, 0x1234);
        cpu.nmi();
        cpu.step();
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sr, 0x23);
        assert_eq!(cpu.sp, 0xfc);
    }

    #[test]
    fn state_after_irq () {
        let mut cpu = test_cpu(Ram::with_capacity(0xffff_u16));
        cpu.sr = 0x23;
        cpu.sp = 0xff;
        cpu.mem.set_le(0xfffe, 0x1234);
        cpu.irq();
        cpu.step();
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sr, 0x27);
        assert_eq!(cpu.sp, 0xfc);
    }

    #[test]
    fn brk_bug () {
        let mut cpu = test_cpu(Ram::with_capacity(0xffff_u16));
        cpu.pc = 0x1000;
        cpu.sr = 0x20;
        cpu.sp = 0xff;
        cpu.mem.set_le(0x1000, 0x00);           // 00: BRK
        cpu.mem.set_le(0x2000, 0x40);           // 40: RTI
        cpu.mem.set_le(0xfffe, 0x2000);
        cpu.irq();
        cpu.step();                             // IRQ happens when BRK is next instruction
        assert_eq!(cpu.pc, 0x2000);             // IRQ is handled
        assert!(!cpu.get_flag(BreakFlag));
        cpu.step();                             // IRQ handler returns
        assert_eq!(cpu.pc, 0x1001);             // BRK was skipped
    }

    #[test]
    fn ruud_baltissen_core_instruction_rom () {
        // Test all instructions using Ruud Baltissen's test ROM from his VHDL 6502 core.
        // See also http://visual6502.org/wiki/index.php?title=6502TestPrograms
        let mut cpu = test_cpu(Ram::with_capacity(0xffff_u16));
        let rom: Rom<u16> = Rom::new(&Path::new("test/ttl6502_v10.rom"));
        for i in range(0_u16, rom.size() as u16) { cpu.mem.set(i + 0xe000, rom.get(i)); }
        cpu.reset();
        for _ in range(0, 3000) {
            cpu.step();
            if cpu.pc == 0xf5b6 { cpu.pc = 0xf5e6; }        // TODO: This skips decimal mode tests for now
        }
        let status = cpu.mem.get(0x0003);
        assert!(status == 0xfe, format!("stopped at ${:04X} with status ${:02X}", cpu.pc, status));
    }
}
