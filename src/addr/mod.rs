//! Generic address handling

pub use self::address::Address;
pub use self::integer::Integer;
pub use self::masked::{Maskable, Masked};

mod address;
mod integer;
mod masked;
