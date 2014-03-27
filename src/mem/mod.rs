pub use self::addr::{Addr, Addressable};
pub use self::ram::Ram;
pub use self::rom::Rom;
pub use self::shared::SharedMemory;

pub mod addr;	// FIXME: why does addr need to be public?
mod ram;
mod rom;
mod shared;
