//!
//! Masked numerics
//!

use std::fmt;
use std::cmp::Ordering;
use std::ops::{Not, BitAnd, BitOr, BitXor};
use addr::Address;

/// Shortcut trait that covers requirements for types that can be masked
trait Maskable: Copy + Not<Output=Self> + BitAnd<Output=Self> + BitOr<Output=Self> { }
impl<T: Maskable> Maskable for T { }

/// Provides a masked numeric, consisting of a numeric value and a bitmask that protects the
/// value. The set bits of the mask prevent changes to same bits of the numeric. This can be
/// useful for masking address pages, i.e. getting the next address of Masked(0x12ff, 0xff00)
/// will result in 0x1200 instead of 0x1300 without the mask.
#[derive(Clone, Copy, Debug)]
pub struct Masked<T>(pub T, pub T);

impl<T> Masked<T> {
    /// Remove the mask and return the unmasked value
    pub fn unmask (self) -> T {
        self.0
    }
}

impl<T: Maskable> Masked<T> {
    /// Map to a new value but protect the masked parts
    pub fn map<F: FnOnce(T) -> T> (self, f: F) -> Masked<T> {
        Masked((self.0 & self.1) | (f(self.0) & !self.1), self.1)
    }
}

impl<T: PartialOrd> PartialOrd<T> for Masked<T> {
    fn partial_cmp (&self, other: &T) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<T: PartialOrd> PartialOrd<Masked<T>> for Masked<T> {
    fn partial_cmp (&self, other: &Masked<T>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord> Ord for Masked<T> {
    fn cmp(&self, other: &Masked<T>) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: PartialEq> PartialEq<T> for Masked<T> {
    fn eq (&self, other: &T) -> bool {
        self.0 == *other
    }
}

impl<T: PartialEq> PartialEq<Masked<T>> for Masked<T> {
    fn eq (&self, other: &Masked<T>) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq> Eq for Masked<T> { }

impl<T: Maskable + Not<Output=T>> Not for Masked<T> {
    type Output = Masked<T>;
    fn not (self) -> Masked<T> {
        self.map(|x| x.not())
    }
}

impl<T: Maskable + BitAnd<Output=T>> BitAnd<T> for Masked<T> {
    type Output = Masked<T>;
    fn bitand (self, other: T) -> Masked<T> {
        self.map(|x| x.bitand(other))
    }
}

impl<T: Maskable + BitAnd<Output=T>> BitAnd<Masked<T>> for Masked<T> {
    type Output = Masked<T>;
    fn bitand (self, other: Masked<T>) -> Masked<T> {
        self.map(|x| x.bitand(other.0))
    }
}

impl<T: Maskable + BitOr<Output=T>> BitOr<T> for Masked<T> {
    type Output = Masked<T>;
    fn bitor (self, other: T) -> Masked<T> {
        self.map(|x| x.bitor(other))
    }
}

impl<T: Maskable + BitOr<Output=T>> BitOr<Masked<T>> for Masked<T> {
    type Output = Masked<T>;
    fn bitor (self, other: Masked<T>) -> Masked<T> {
        self.map(|x| x.bitor(other.0))
    }
}

impl<T: Maskable + BitXor<Output=T>> BitXor<T> for Masked<T> {
    type Output = Masked<T>;
    fn bitxor (self, other: T) -> Masked<T> {
        self.map(|x| x.bitxor(other))
    }
}

impl<T: Maskable + BitXor<Output=T>> BitXor<Masked<T>> for Masked<T> {
    type Output = Masked<T>;
    fn bitxor (self, other: Masked<T>) -> Masked<T> {
        self.map(|x| x.bitxor(other.0))
    }
}

impl<T: fmt::UpperHex> fmt::UpperHex for Masked<T> {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<A: Maskable + Address> Address for Masked<A> {
    fn zero () -> Masked<A> {
        Masked(A::zero(), A::zero())
    }

    fn to_u16 (&self) -> u16 {
        self.0.to_u16()
    }

    fn offset (&self, offset: i16) -> Masked<A> {
        self.map(|addr| addr.offset(offset))
    }
}


#[cfg(test)]
mod tests {
    use addr::Address;
    use super::*;

    #[test]
    fn mask_and_unmask () {
        assert_eq!(Masked(0x1234, 0xff00).unmask(), 0x1234);
    }

    #[test]
    fn mapping () {
        assert_eq!(Masked(0x1234, 0xff00).map(|_| 0), 0x1200);
    }

    #[test]
    fn ord_and_eq () {
        assert!(Masked(0x12ff, 0xff00) < 0x1300);
        assert!(Masked(0x12ff, 0xff00) < Masked(0x1300, 0xfff0));
        assert_eq!(Masked(0x12ff, 0xff00), 0x12ff);
        assert_eq!(Masked(0x12ff, 0xff00), Masked(0x12ff, 0xfff0));
    }

    #[test]
    fn boolean_ops () {
        let value = Masked(0b1100110011001100_u16, 0b1111000011110000);
        assert_eq!(!value, 0b1100001111000011);
        assert_eq!( value & 0b1111111100000000, 0b1100110011000000);
        assert_eq!( value | 0b1111111100000000, 0b1100111111001100);
        assert_eq!( value ^ 0b1111111100000000, 0b1100001111001100);
    }

    #[test]
    fn address_offset () {
        assert_eq!(Masked(0x12ff, 0x0000).offset(1), 0x1300);
        assert_eq!(Masked(0x12ff, 0xff00).offset(1), 0x1200);
        assert_eq!(Masked(0x12ff, 0xfff0).offset(1), 0x12f0);
        assert_eq!(Masked(0x1300, 0x0000).offset(-1), 0x12ff);
        assert_eq!(Masked(0x1300, 0xff00).offset(-1), 0x13ff);
        assert_eq!(Masked(0x1300, 0xfff0).offset(-1), 0x130f);
    }

    #[test]
    fn address_iterating () {
        let mut it = Masked(0x12fe, 0xff00).successive();
        assert_eq!(it.next().unwrap(), 0x12fe);
        assert_eq!(it.next().unwrap(), 0x12ff);
        assert_eq!(it.next().unwrap(), 0x1200);
        assert_eq!(it.next().unwrap(), 0x1201);
    }
}
