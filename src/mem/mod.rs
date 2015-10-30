//!
//! Generic addressing (memory)
//!

pub use self::addressable::Addressable;
pub use self::ram::Ram;
pub use self::rom::Rom;

pub mod addressable;        // FIXME: needs to be pub, see Rust issue #18241 and #16264
mod ram;
mod rom;
mod shared;

#[cfg(test)]
pub mod test;
