pub use self::addr::{Addr, Addressable};
pub use self::ram::Ram;
pub use self::rom::Rom;

pub mod addr;
pub mod ram;
pub mod rom;

#[cfg(test)]
pub mod test;
