//!
//! Generic addresses
//!

use std::{fmt, mem};
use std::ops::{Not, BitAnd, BitOr, BitXor};

/// A trait for all addresses
pub trait Address: Copy + Ord + Eq + Not<Output=Self> + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self> + fmt::UpperHex {
    /// Type of address offset
    type Offset;

    /// The zero address
    /// TODO: Use std::num::Zero instead when it becomes stable
    fn zero () -> Self;

    /// The succeeding address (wrapping)
    fn next (self) -> Self;

    /// Calculate new address with given offset (wrapping)
    fn offset (self, offset: Self::Offset) -> Self;

    /// Calculate new address with given offset (wrapping) while protecting the masked part of
    /// the address
    fn offset_masked (&self, offset: Self::Offset, mask: Self) -> Self {
        (*self & mask) | (self.offset(offset) & !mask)
    }

    /// Return an iterator for getting successive addresses (wrapping)
    fn successive (self) -> Iter<Self> {
        Iter { addr: self }
    }

    /// Return an object for displaying the address
    fn display (&self) -> Display<Self> {
        Display { addr: self }
    }
}

macro_rules! impl_address {
    ($addr_type:ty, $offset_type:ty) => (
        impl Address for $addr_type {
            type Offset = $offset_type;

            fn zero () -> $addr_type { 0 }

            fn next (self) -> $addr_type { self.wrapping_add(1) }

            fn offset (self, offset: $offset_type) -> $addr_type {
                if offset < 0 {
                    self.wrapping_sub((-offset as $addr_type))
                } else {
                    self.wrapping_add(offset as $addr_type)
                }
            }
        }
    );
}

impl_address!( u8,  i8);
impl_address!(u16, i16);
impl_address!(u32, i32);

/// Iterator for getting successive addresses
pub struct Iter<A> {
    addr: A,
}

impl<A: Address> Iterator for Iter<A> {
    type Item = A;

    fn next (&mut self) -> Option<A> {
        let addr = self.addr;
        self.addr = self.addr.next();
        Some(addr)
    }
}

/// Helper struct for displaying an address
pub struct Display<'a, A: 'a> {
    addr: &'a A,
}

impl<'a, A: Address> fmt::Display for Display<'a, A> {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        match mem::size_of::<A>() {
            1 => write!(f, "${:02X}", self.addr),
            2 => write!(f, "${:04X}", self.addr),
            4 => write!(f, "${:08X}", self.addr),
            _ => write!(f, "${:X}", self.addr),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero () {
        assert_eq!(u16::zero(), 0x0000);
    }

    #[test]
    fn next () {
        assert_eq!(0x1234_u16.next(), 0x1235);
        assert_eq!(0xffff_u16.next(), 0x0000);
    }

    #[test]
    fn offset () {
        assert_eq!(0x1234_u16.offset( 5), 0x1239);
        assert_eq!(0x1234_u16.offset(-3), 0x1231);
    }

    #[test]
    fn offset_wrapping () {
        assert_eq!(0xffff_u16.offset( 1), 0x0000);
        assert_eq!(0x0000_u16.offset(-1), 0xffff);
    }

    #[test]
    fn offset_masked () {
        assert_eq!(0x12ff_u16.offset_masked( 1, 0x0000), 0x1300);
        assert_eq!(0x12ff_u16.offset_masked( 1, 0xff00), 0x1200);
        assert_eq!(0x1300_u16.offset_masked(-1, 0x0000), 0x12ff);
        assert_eq!(0x1300_u16.offset_masked(-1, 0xff00), 0x13ff);
    }

    #[test]
    fn iterating () {
        let mut it = 0xfffe_u16.successive();
        assert_eq!(it.next(), Some(0xfffe));
        assert_eq!(it.next(), Some(0xffff));
        assert_eq!(it.next(), Some(0x0000));
        assert_eq!(it.next(), Some(0x0001));
    }

    #[test]
    fn displaying () {
        assert_eq!(format!("{}", 0x0f_u8.display()), "$0F");
        assert_eq!(format!("{}", 0x01ff_u16.display()), "$01FF");
    }
}
