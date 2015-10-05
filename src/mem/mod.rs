//!
//! Address/memory handling
//!

pub use self::address::Address;
pub use self::addressable::Addressable;
pub use self::ram::Ram;
pub use self::rom::Rom;

mod address;
mod addressable;
mod ram;
mod rom;
mod shared;

#[cfg(test)]
pub mod test;
