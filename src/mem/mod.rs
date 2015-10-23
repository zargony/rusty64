//!
//! Generic addressing (memory)
//!

pub use self::addressable::Addressable;
pub use self::ram::Ram;
pub use self::rom::Rom;

mod addressable;
mod ram;
mod rom;
mod shared;

#[cfg(test)]
pub mod test;
