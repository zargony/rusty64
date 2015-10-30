//!
//! Generic addressing
//!

use std::{fmt, mem};
use addr::{Address, Integer};

/// A trait for anything that has an address bus and can get/set data. The address (any type that
/// implements the `Address` trait) is 16 bit always. The data that can be get/set is 8 bit.
pub trait Addressable {
    /// Memory read: returns the data at the given address
    fn get<A: Address> (&self, addr: A) -> u8;

    /// Get a type from the given address by copying data bits. This gets a number in host platform
    /// byte order format. Since this unsafely transmutes to the target type, this method is
    /// marked unsafe. For integers, better use `get_be` or `get_le` instead.
    unsafe fn get_type<A: Address, T: Copy> (&self, addr: A) -> T {
        let mut val: T = mem::uninitialized();
        let ptr = &mut val as *mut T as *mut u8;
        for (i, addr) in addr.successive().take(mem::size_of::<T>()).enumerate() {
            *ptr.offset(i as isize) = self.get(addr);
        }
        val
    }

    /// Get a number in big endian format from the given address
    fn get_be<A: Address, T: Integer> (&self, addr: A) -> T {
        unsafe { T::from_be(self.get_type(addr)) }
    }

    /// Get a number in little endian format from the given address
    fn get_le<A: Address, T: Integer> (&self, addr: A) -> T {
        unsafe { T::from_le(self.get_type(addr)) }
    }

    /// Memory write: set the data at the given address
    fn set<A: Address> (&mut self, addr: A, data: u8);

    /// Store a type to the given address by copying data bits. This stores a type in host
    /// platform byte order format. Since this unsafely transmutes the stored type, this method
    /// is marked unsafe. For integers, better use `set_be` or `set_le` instead.
    unsafe fn set_type<A: Address, T: Copy> (&mut self, addr: A, val: T) {
        let ptr = &val as *const T as *const u8;
        for (i, addr) in addr.successive().take(mem::size_of::<T>()).enumerate() {
            self.set(addr, *ptr.offset(i as isize));
        }
    }

    /// Store a number in big endian format to the given address
    fn set_be<A: Address, T: Integer> (&mut self, addr: A, val: T) {
        unsafe { self.set_type(addr, T::to_be(val)); }
    }

    /// Store a number in little endian format to the given address
    fn set_le<A: Address, T: Integer> (&mut self, addr: A, val: T) {
        unsafe { self.set_type(addr, T::to_le(val)); }
    }

    /// Copy data from another addressable source
    fn copy<A1: Address, A2: Address, M: Addressable> (&mut self, self_addr: A1, other: &M, other_addr: A2, size: usize) {
        for (dst_addr, src_addr) in self_addr.successive().zip(other_addr.successive()).take(size) {
            self.set(dst_addr, other.get(src_addr));
        }
    }

    /// Return an object for displaying a hexdump of the given address range
    fn hexdump<A: Address> (&self, addr1: A, addr2: A) -> HexDump<A, Self> {
        HexDump { mem: self, addr1: addr1, addr2: addr2 }
    }
}

/// Helper struct for displaying a hexdump of an address range
pub struct HexDump<'a, A, M: 'a + ?Sized> {
    mem: &'a M,
    addr1: A,
    addr2: A,
}

impl<'a, A: Address, M: Addressable> fmt::Display for HexDump<'a, A, M> {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        for addr in self.addr1.successive().upto(self.addr2) {
            try!(write!(f, "{:02X} ", self.mem.get(addr)));
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use addr::Masked;
    use mem::test::TestMemory;
    use super::*;

    #[test]
    fn get_byte () {
        let data = TestMemory;
        assert_eq!(data.get(0x0012), 0x12);
        assert_eq!(data.get(0x1234), 0x46);
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
        assert_eq!(    0x1112_u16, data.get_be(Masked(0x12ff, 0xff00)));
        assert_eq!(0x10111213_u32, data.get_be(Masked(0x12fe, 0xff00)));
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
        assert_eq!(    0x1211_u16, data.get_le(Masked(0x12ff, 0xff00)));
        assert_eq!(0x13121110_u32, data.get_le(Masked(0x12fe, 0xff00)));
    }

    #[test]
    fn set_byte () {
        let mut data = TestMemory;
        data.set(0x0012, 0x12);
        data.set(0x1234, 0x46);
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
        data.set_be(Masked(0x12ff, 0xff00),     0x1112_u16);
        data.set_be(Masked(0x12fe, 0xff00), 0x10111213_u32);
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
        data.set_le(Masked(0x12ff, 0xff00),     0x1211_u16);
        data.set_le(Masked(0x12fe, 0xff00), 0x13121110_u32);
    }

    #[test]
    fn copying_memory () {
        let data1 = TestMemory;
        let mut data2 = TestMemory;
        data2.copy(0x8000, &data1, 0x0080, 0x0080);
    }

    #[test]
    fn dumping_memory () {
        let data = TestMemory;
        assert_eq!(format!("{}", data.hexdump(0x0100, 0x0100)), "01 ");
        assert_eq!(format!("{}", data.hexdump(0x0100, 0x0101)), "01 02 ");
        assert_eq!(format!("{}", data.hexdump(0x0100, 0x0105)), "01 02 03 04 05 06 ");
    }
}
