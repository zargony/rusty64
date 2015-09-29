//!
//! Address/memory handling
//!

pub use self::addr::{Address, Addressable};
pub use self::ram::Ram;
pub use self::rom::Rom;

mod addr;
mod ram;
mod rom;
mod shared;

#[cfg(test)]
pub mod test;
