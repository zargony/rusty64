pub use self::cpu::CPU;
#[cfg(c64)]
pub use self::mos6510::Mos6510;

mod cpu;
#[cfg(test)] #[cfg(c64)]
mod mos6502;
#[cfg(test)] #[cfg(c64)]
mod mos6510;
