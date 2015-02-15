pub use self::addr::{Address, Addressable};
pub use self::ram::Ram;
pub use self::rom::Rom;

pub mod addr;       // FIXME: needs to be pub, see Rust issue #18241 and #16264
mod ram;
mod rom;
mod shared;
