//!
//! Generic addresses
//!

use std::{fmt, mem};

/// A trait for all 16-bit address types
pub trait Address: Copy + Ord + Eq + fmt::UpperHex {
    /// The zero address
    /// TODO: Use std::num::Zero instead when it becomes stable
    fn zero () -> Self;

    /// The address as an unsigned integer
    fn to_u16 (&self) -> u16;

    /// Calculate new address with given offset (wrapping)
    fn offset (&self, offset: i16) -> Self;

    /// Return an iterator for getting successive addresses (wrapping)
    fn successive (&self) -> Iter<Self> {
        Iter { addr: *self }
    }

    /// Return an object for displaying the address
    fn display (&self) -> Display<Self> {
        Display { addr: self }
    }
}

impl Address for u16 {
    fn zero () -> u16 { 0 }

    fn to_u16 (&self) -> u16 { *self }

    fn offset (&self, offset: i16) -> u16 {
        if offset < 0 {
            self.wrapping_sub(-offset as u16)
        } else {
            self.wrapping_add(offset as u16)
        }
    }
}

/// Iterator for getting successive addresses
pub struct Iter<A> {
    addr: A,
}

impl<A: Address> Iter<A> {
    /// Bound iterator to only run until the given last address
    pub fn upto (self, last_addr: A) -> UpTo<A> {
        UpTo { it: self, last_addr: last_addr, flag: false }
    }
}

impl<A: Address> Iterator for Iter<A> {
    type Item = A;

    fn next (&mut self) -> Option<A> {
        let addr = self.addr;
        self.addr = self.addr.offset(1);
        Some(addr)
    }
}

/// Iterator for getting successive addresses until a given last address
pub struct UpTo<A> {
    it: Iter<A>,
    last_addr: A,
    flag: bool,
}

impl<A: Address> Iterator for UpTo<A> {
    type Item = A;

    fn next (&mut self) -> Option<A> {
        if self.flag {
            None
        } else {
            self.it.next().and_then(|x| {
                if x == self.last_addr { self.flag = true; }
                Some(x)
            })
        }
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
    fn offset () {
        assert_eq!(0x1234.offset( 5), 0x1239);
        assert_eq!(0x1234.offset(-3), 0x1231);
    }

    #[test]
    fn offset_wrapping () {
        assert_eq!(0xffff.offset( 1), 0x0000);
        assert_eq!(0x0000.offset(-1), 0xffff);
    }

    #[test]
    fn iterating () {
        let mut it = 0xfffe.successive();
        assert_eq!(it.next(), Some(0xfffe));
        assert_eq!(it.next(), Some(0xffff));
        assert_eq!(it.next(), Some(0x0000));
        assert_eq!(it.next(), Some(0x0001));
    }

    #[test]
    fn bounded_iterating () {
        let mut it = 0xfffe.successive().upto(0xffff);
        assert_eq!(it.next(), Some(0xfffe));
        assert_eq!(it.next(), Some(0xffff));
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn displaying () {
        assert_eq!(format!("{}", 0x01ff.display()), "$01FF");
    }
}