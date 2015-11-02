//!
//! Generic addresses
//!

use std::{fmt, mem};

/// A trait for all 16-bit address types
pub trait Address: Copy + Ord + Eq + fmt::UpperHex {
    /// Calculate new address with given offset (wrapping)
    fn offset (&self, offset: i16) -> Self;

    /// The address as an unsigned integer
    fn to_u16 (&self) -> u16;

    /// Return an object for displaying the address
    fn display (&self) -> Display<Self> {
        Display { addr: self }
    }
}

impl Address for u16 {
    fn to_u16 (&self) -> u16 { *self }

    fn offset (&self, offset: i16) -> u16 {
        if offset < 0 {
            self.wrapping_sub(-offset as u16)
        } else {
            self.wrapping_add(offset as u16)
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
    fn displaying () {
        assert_eq!(format!("{}", 0x01ff.display()), "$01FF");
    }
}
