//! Generic addressing

use crate::addr::{Address, Integer};
use std::fmt::{self, Write};

/// A trait for anything that has an address bus and can get/set data. The address (any type that
/// implements the `Address` trait) is 16 bit always. The data that can be get/set is 8 bit.
pub trait Addressable {
    /// Memory read: returns the data at the given address
    fn get<A: Address>(&self, addr: A) -> u8;

    /// Memory read: returns the data bytes at the given address
    fn getn<A: Address, const N: usize>(&self, addr: A) -> [u8; N] {
        let mut bytes = [0; N];
        for (offset, byte) in bytes.iter_mut().enumerate() {
            *byte = self.get(addr.offset(offset as i16))
        }
        bytes
    }

    /// Get a number in big endian format from the given address
    fn get_be<A: Address, const N: usize, T: Integer<N>>(&self, addr: A) -> T {
        T::from_be_bytes(&self.getn(addr))
    }

    /// Get a number in little endian format from the given address
    fn get_le<A: Address, const N: usize, T: Integer<N>>(&self, addr: A) -> T {
        T::from_le_bytes(&self.getn(addr))
    }

    /// Memory write: set the data at the given address
    fn set<A: Address>(&mut self, addr: A, data: u8);

    /// Memory write: set the data bytes at the given address
    fn setn<A: Address, const N: usize>(&mut self, addr: A, bytes: [u8; N]) {
        for (offset, byte) in bytes.iter().enumerate() {
            self.set(addr.offset(offset as i16), *byte);
        }
    }

    /// Store a number in big endian format to the given address
    fn set_be<A: Address, const N: usize, T: Integer<N>>(&mut self, addr: A, val: T) {
        self.setn(addr, val.to_be_bytes())
    }

    /// Store a number in little endian format to the given address
    fn set_le<A: Address, const N: usize, T: Integer<N>>(&mut self, addr: A, val: T) {
        self.setn(addr, val.to_le_bytes())
    }

    /// Copy data from another addressable source
    fn copy<A1: Address, A2: Address, M: Addressable>(
        &mut self,
        self_addr: A1,
        other: &M,
        other_addr: A2,
        size: usize,
    ) {
        for i in 0..size {
            self.set(
                self_addr.offset(i as i16),
                other.get(other_addr.offset(i as i16)),
            );
        }
    }

    /// Return an object for displaying a hexdump of the given address range
    fn hexdump<A: Address, I: Iterator<Item = A> + Clone>(&self, iter: I) -> HexDump<I, Self> {
        HexDump { mem: self, iter }
    }
}

/// Helper struct for displaying a hexdump of an address range
pub struct HexDump<'a, I, M: 'a + ?Sized> {
    mem: &'a M,
    iter: I,
}

impl<'a, A: Address, I: Iterator<Item = A> + Clone, M: Addressable> fmt::Display
    for HexDump<'a, I, M>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        let mut iter = self.iter.clone().peekable();
        while let Some(addr) = iter.next() {
            write!(str, "{:02X}", self.mem.get(addr))?;
            if iter.peek().is_some() {
                write!(str, " ")?;
            }
        }
        str.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test::TestMemory;
    use super::*;
    use crate::addr::Masked;

    #[test]
    fn get_byte() {
        let data = TestMemory;
        assert_eq!(data.get(0x0012), 0x12);
        assert_eq!(data.get(0x1234), 0x46);
    }

    #[test]
    fn get_bytes() {
        let data = TestMemory;
        assert_eq!(data.getn::<_, 4>(0x0012), [0x12, 0x13, 0x14, 0x15]);
        assert_eq!(data.getn::<_, 4>(0x1234), [0x46, 0x47, 0x48, 0x49]);
    }

    #[test]
    fn get_big_endian_number() {
        let data = TestMemory;
        assert_eq!(0x02_u8, data.get_be(0x0002));
        assert_eq!(0x54_u8, data.get_be(0x0054));
        assert_eq!(0x0203_u16, data.get_be(0x0002));
        assert_eq!(0x5455_u16, data.get_be(0x0054));
        assert_eq!(0x02030405_u32, data.get_be(0x0002));
        assert_eq!(0x54555657_u32, data.get_be(0x0054));
    }

    #[test]
    fn get_signed_big_endian_number() {
        let data = TestMemory;
        assert_eq!(0x54_i8, data.get_be(0x0054));
        assert_eq!(-0x5b_i8, data.get_be(0x00a5));
        assert_eq!(0x5455_i16, data.get_be(0x0054));
        assert_eq!(-0x5a5a_i16, data.get_be(0x00a5));
        assert_eq!(0x54555657_i32, data.get_be(0x0054));
        assert_eq!(-0x5a595858_i32, data.get_be(0x00a5));
    }

    #[test]
    fn get_masked_big_endian_number() {
        let data = TestMemory;
        assert_eq!(0x1112_u16, data.get_be(Masked(0x12ff, 0xff00)));
        assert_eq!(0x10111213_u32, data.get_be(Masked(0x12fe, 0xff00)));
    }

    #[test]
    fn get_little_endian_number() {
        let data = TestMemory;
        assert_eq!(0x02_u8, data.get_le(0x0002));
        assert_eq!(0x54_u8, data.get_le(0x0054));
        assert_eq!(0x0302_u16, data.get_le(0x0002));
        assert_eq!(0x5554_u16, data.get_le(0x0054));
        assert_eq!(0x05040302_u32, data.get_le(0x0002));
        assert_eq!(0x57565554_u32, data.get_le(0x0054));
    }

    #[test]
    fn get_signed_little_endian_number() {
        let data = TestMemory;
        assert_eq!(0x54_i8, data.get_le(0x0054));
        assert_eq!(-0x5b_i8, data.get_le(0x00a5));
        assert_eq!(0x5554_i16, data.get_le(0x0054));
        assert_eq!(-0x595b_i16, data.get_le(0x00a5));
        assert_eq!(0x57565554_i32, data.get_le(0x0054));
        assert_eq!(-0x5758595b_i32, data.get_le(0x00a5));
    }

    #[test]
    fn get_masked_little_endian_number() {
        let data = TestMemory;
        assert_eq!(0x1211_u16, data.get_le(Masked(0x12ff, 0xff00)));
        assert_eq!(0x13121110_u32, data.get_le(Masked(0x12fe, 0xff00)));
    }

    #[test]
    fn set_byte() {
        let mut data = TestMemory;
        data.set(0x0012, 0x12);
        data.set(0x1234, 0x46);
    }

    #[test]
    fn set_bytes() {
        let mut data = TestMemory;
        data.setn::<_, 4>(0x0012, [0x12, 0x13, 0x14, 0x15]);
        data.setn::<_, 4>(0x1234, [0x46, 0x47, 0x48, 0x49]);
    }

    #[test]
    fn set_big_endian_number() {
        let mut data = TestMemory;
        data.set_be(0x0002, 0x02_u8);
        data.set_be(0x0054, 0x54_u8);
        data.set_be(0x0002, 0x0203_u16);
        data.set_be(0x0054, 0x5455_u16);
        data.set_be(0x0002, 0x02030405_u32);
        data.set_be(0x0054, 0x54555657_u32);
    }

    #[test]
    fn set_signed_big_endian_number() {
        let mut data = TestMemory;
        data.set_be(0x0054, 0x54_i8);
        data.set_be(0x00a5, -0x5b_i8);
        data.set_be(0x0054, 0x5455_i16);
        data.set_be(0x00a5, -0x5a5a_i16);
        data.set_be(0x0054, 0x54555657_i32);
        data.set_be(0x00a5, -0x5a595858_i32);
    }

    #[test]
    fn set_masked_big_endian_number() {
        let mut data = TestMemory;
        data.set_be(Masked(0x12ff, 0xff00), 0x1112_u16);
        data.set_be(Masked(0x12fe, 0xff00), 0x10111213_u32);
    }

    #[test]
    fn set_little_endian_number() {
        let mut data = TestMemory;
        data.set_le(0x0002, 0x02_u8);
        data.set_le(0x0054, 0x54_u8);
        data.set_le(0x0002, 0x0302_u16);
        data.set_le(0x0054, 0x5554_u16);
        data.set_le(0x0002, 0x05040302_u32);
        data.set_le(0x0054, 0x57565554_u32);
    }

    #[test]
    fn set_signed_little_endian_number() {
        let mut data = TestMemory;
        data.set_le(0x0054, 0x54_i8);
        data.set_le(0x00a5, -0x5b_i8);
        data.set_le(0x0054, 0x5554_i16);
        data.set_le(0x00a5, -0x595b_i16);
        data.set_le(0x0054, 0x57565554_i32);
        data.set_le(0x00a5, -0x5758595b_i32);
    }

    #[test]
    fn set_masked_little_endian_number() {
        let mut data = TestMemory;
        data.set_le(Masked(0x12ff, 0xff00), 0x1211_u16);
        data.set_le(Masked(0x12fe, 0xff00), 0x13121110_u32);
    }

    #[test]
    fn copying_memory() {
        let data1 = TestMemory;
        let mut data2 = TestMemory;
        data2.copy(0x8000, &data1, 0x0080, 0x0080);
    }

    #[test]
    fn dumping_memory() {
        let data = TestMemory;
        assert_eq!(format!("{}", data.hexdump(0x0100..0x0101)), "01");
        assert_eq!(format!("{}", data.hexdump(0x0100..0x0102)), "01 02");
        assert_eq!(format!("{}", data.hexdump(0x0100..0x0104)), "01 02 03 04");
        assert_eq!(
            format!("{:16}", data.hexdump(0x0100..0x0104)),
            "01 02 03 04     ",
        );
        assert_eq!(
            format!("{:>16}", data.hexdump(0x0100..0x0104)),
            "     01 02 03 04",
        );
    }
}
