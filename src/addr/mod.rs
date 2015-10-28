//!
//! Generic address handling
//!

pub use self::address::Address;
pub use self::masked::Masked;

pub mod address;            // FIXME: needs to be pub, see Rust issue #18241 and #16264
mod masked;
