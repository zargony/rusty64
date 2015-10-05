//!
//! Generic addressing
//!

use std::{fmt, mem};
use num::{self, PrimInt};
use super::Address;

/// A trait for anything that has an address bus and can get/set data. The data size that can be
/// get/set is u8 always, the address size is given as a type parameter and can be of any size
/// (typically u16 or u32).
pub trait Addressable<A: Address> {
    /// Memory read: returns the data at the given address
    fn get (&self, addr: A) -> u8;

    /// Get a number in host platform byte order format from the given address. Note: Don't use
    /// this directly, better use `get_be` or `get_le` instead.
    fn get_number<T: PrimInt> (&self, addr: A, mask: A) -> T {
        let mut val: T = T::zero();
        let size = mem::size_of::<T>() as isize;
        let ptr = &mut val as *mut T as *mut u8;
        for i in 0..size {
            unsafe { *ptr.offset(i) = self.get(addr.offset_masked(i, mask)); }
        }
        val
    }

    /// Get a number in big endian format from the given address
    fn get_be<T: PrimInt> (&self, addr: A) -> T {
        T::from_be(self.get_number(addr, A::zero()))
    }

    /// Get a number in big endian format from the given masked address
    fn get_be_masked<T: PrimInt> (&self, addr: A, mask: A) -> T {
        T::from_be(self.get_number(addr, mask))
    }

    /// Get a number in little endian format from the given address
    fn get_le<T: PrimInt> (&self, addr: A) -> T {
        T::from_le(self.get_number(addr, A::zero()))
    }

    /// Get a number in little endian format from the given masked address
    fn get_le_masked<T: PrimInt> (&self, addr: A, mask: A) -> T {
        T::from_le(self.get_number(addr, mask))
    }

    /// Memory write: set the data at the given address
    fn set (&mut self, addr: A, data: u8);

    /// Store a number in host platform byte order format to the given address. Note: Don't use
    /// this directly, better use `set_be` or `set_le` instead.
    fn set_number<T: PrimInt> (&mut self, addr: A, mask: A, val: T) {
        let size = mem::size_of::<T>() as isize;
        let ptr = &val as *const T as *const u8;
        for i in 0..size {
            unsafe { self.set(addr.offset_masked(i, mask), *ptr.offset(i)); }
        }
    }

    /// Store a number in big endian format to the given address
    fn set_be<T: PrimInt> (&mut self, addr: A, val: T) {
        self.set_number(addr, A::zero(), T::to_be(val));
    }

    /// Store a number in big endian format to the given masked address
    fn set_be_masked<T: PrimInt> (&mut self, addr: A, mask: A, val: T) {
        self.set_number(addr, mask, T::to_be(val));
    }

    /// Store a number in little endian format to the given address
    fn set_le<T: PrimInt> (&mut self, addr: A, val: T) {
        self.set_number(addr, A::zero(), T::to_le(val));
    }

    /// Store a number in little endian format to the given masked address
    fn set_le_masked<T: PrimInt> (&mut self, addr: A, mask: A, val: T) {
        self.set_number(addr, mask, T::to_le(val));
    }

    /// Copy data from another addressable source
    fn copy<M: Addressable<A>> (&mut self, self_addr: A, other: &M, other_addr: A, size: usize) {
        for i in 0..size {
            self.set(self_addr.offset(i), other.get(other_addr.offset(i)))
        }
    }

    /// Return an object for displaying a hexdump of the given address range
    fn hexdump (&self, addr1: A, addr2: A) -> HexDump<A, Self> {
        HexDump { mem: self, addr1: addr1, addr2: addr2 }
    }
}

/// Helper object for displaying a hexdump of an address range
pub struct HexDump<'a, A, M: 'a + ?Sized> {
    mem: &'a M,
    addr1: A,
    addr2: A,
}

impl<'a, A: Address, M: Addressable<A>> fmt::Display for HexDump<'a, A, M> {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        for addr in num::range(self.addr1, self.addr2) {
            try!(write!(f, "{:02X} ", self.mem.get(addr)));
        }
        write!(f, "{:02X}", self.mem.get(self.addr2))
    }
}


#[cfg(test)]
mod tests {
    use super::super::test::TestMemory;
    use super::*;

    #[test]
    fn get_byte () {
        let data = TestMemory;
        assert_eq!(data.get(0x0012), 0x12);
        assert_eq!(data.get(0x1234), 0x34);
    }

    #[test]
    fn get_big_endian_number () {
        let data = TestMemory;
        assert_eq!(      0x02_u8 , data.get_be(0x0002));
        assert_eq!(      0x54_u8 , data.get_be(0x0054));
        assert_eq!(    0x0203_u16, data.get_be(0x0002));
        assert_eq!(    0x5455_u16, data.get_be(0x0054));
        assert_eq!(0x02030405_u32, data.get_be(0x0002));
        assert_eq!(0x54555657_u32, data.get_be(0x0054));
    }

    #[test]
    fn get_signed_big_endian_number () {
        let data = TestMemory;
        assert_eq!(       0x54_i8 , data.get_be(0x0054));
        assert_eq!(      -0x5b_i8 , data.get_be(0x00a5));
        assert_eq!(     0x5455_i16, data.get_be(0x0054));
        assert_eq!(    -0x5a5a_i16, data.get_be(0x00a5));
        assert_eq!( 0x54555657_i32, data.get_be(0x0054));
        assert_eq!(-0x5a595858_i32, data.get_be(0x00a5));
    }

    #[test]
    fn get_masked_big_endian_number () {
        let data = TestMemory;
        assert_eq!(    0xff00_u16, data.get_be_masked(0x12ff, 0xff00));
        assert_eq!(0xfeff0001_u32, data.get_be_masked(0x12fe, 0xff00));
    }

    #[test]
    fn get_little_endian_number () {
        let data = TestMemory;
        assert_eq!(      0x02_u8 , data.get_le(0x0002));
        assert_eq!(      0x54_u8 , data.get_le(0x0054));
        assert_eq!(    0x0302_u16, data.get_le(0x0002));
        assert_eq!(    0x5554_u16, data.get_le(0x0054));
        assert_eq!(0x05040302_u32, data.get_le(0x0002));
        assert_eq!(0x57565554_u32, data.get_le(0x0054));
    }

    #[test]
    fn get_signed_little_endian_number () {
        let data = TestMemory;
        assert_eq!(       0x54_i8 , data.get_le(0x0054));
        assert_eq!(      -0x5b_i8 , data.get_le(0x00a5));
        assert_eq!(     0x5554_i16, data.get_le(0x0054));
        assert_eq!(    -0x595b_i16, data.get_le(0x00a5));
        assert_eq!( 0x57565554_i32, data.get_le(0x0054));
        assert_eq!(-0x5758595b_i32, data.get_le(0x00a5));
    }

    #[test]
    fn get_masked_little_endian_number () {
        let data = TestMemory;
        assert_eq!(    0x00ff_u16, data.get_le_masked(0x12ff, 0xff00));
        assert_eq!(0x0100fffe_u32, data.get_le_masked(0x12fe, 0xff00));
    }

    #[test]
    fn set_byte () {
        let mut data = TestMemory;
        data.set(0x0012, 0x12);
        data.set(0x1234, 0x34);
    }

    #[test]
    fn set_big_endian_number () {
        let mut data = TestMemory;
        data.set_be(0x0002,       0x02_u8 );
        data.set_be(0x0054,       0x54_u8 );
        data.set_be(0x0002,     0x0203_u16);
        data.set_be(0x0054,     0x5455_u16);
        data.set_be(0x0002, 0x02030405_u32);
        data.set_be(0x0054, 0x54555657_u32);
    }

    #[test]
    fn set_signed_big_endian_number () {
        let mut data = TestMemory;
        data.set_be(0x0054,        0x54_i8 );
        data.set_be(0x00a5,       -0x5b_i8 );
        data.set_be(0x0054,      0x5455_i16);
        data.set_be(0x00a5,     -0x5a5a_i16);
        data.set_be(0x0054,  0x54555657_i32);
        data.set_be(0x00a5, -0x5a595858_i32);
    }

    #[test]
    fn set_masked_big_endian_number () {
        let mut data = TestMemory;
        data.set_be_masked(0x12ff, 0xff00,     0xff00_u16);
        data.set_be_masked(0x12fe, 0xff00, 0xfeff0001_u32);
    }

    #[test]
    fn set_little_endian_number () {
        let mut data = TestMemory;
        data.set_le(0x0002,       0x02_u8 );
        data.set_le(0x0054,       0x54_u8 );
        data.set_le(0x0002,     0x0302_u16);
        data.set_le(0x0054,     0x5554_u16);
        data.set_le(0x0002, 0x05040302_u32);
        data.set_le(0x0054, 0x57565554_u32);
    }

    #[test]
    fn set_signed_little_endian_number () {
        let mut data = TestMemory;
        data.set_le(0x0054,        0x54_i8 );
        data.set_le(0x00a5,       -0x5b_i8 );
        data.set_le(0x0054,      0x5554_i16);
        data.set_le(0x00a5,     -0x595b_i16);
        data.set_le(0x0054,  0x57565554_i32);
        data.set_le(0x00a5, -0x5758595b_i32);
    }

    #[test]
    fn set_masked_little_endian_number () {
        let mut data = TestMemory;
        data.set_le_masked(0x12ff, 0xff00,     0x00ff_u16);
        data.set_le_masked(0x12fe, 0xff00, 0x0100fffe_u32);
    }

    #[test]
    fn copying_memory () {
        let data1 = TestMemory;
        let mut data2 = TestMemory;
        data2.copy(0x8000, &data1, 0x0000, 0x1000);
    }

    #[test]
    fn dumping_memory () {
        let data = TestMemory;
        assert_eq!(format!("{}", data.hexdump(0x0100, 0x0100)), "00");
        assert_eq!(format!("{}", data.hexdump(0x0100, 0x0101)), "00 01");
        assert_eq!(format!("{}", data.hexdump(0x0100, 0x0105)), "00 01 02 03 04 05");
    }
}
