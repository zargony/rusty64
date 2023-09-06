//! CPU handling

pub use self::cpu::Cpu;
pub use self::mos6502::Mos6502;
pub use self::mos6510::Mos6510;

#[allow(clippy::module_inception)]
mod cpu;
mod mos6502;
mod mos6510;
