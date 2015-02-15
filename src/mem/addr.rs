use std::{fmt, mem};
use std::num::{self, Int, UnsignedInt};

/// A trait for all addresses
pub trait Address: UnsignedInt + fmt::UpperHex {
    /// Calculate new address with given offset
    fn offset<T: Int> (self, offset: T) -> Self {
        if offset < Int::zero() {
            self - num::cast(Int::zero() - offset).unwrap()
        } else {
            self + num::cast(offset).unwrap()
        }
    }

    /// Calculate new address with given offset while protecting the masked part of the address
    fn offset_masked<T: Int> (self, offset: T, mask: Self) -> Self {
        (self & mask) | (self.offset(offset) & !mask)
    }

    /// Return an object for displaying the address
    fn display (self) -> AddressDisplay<Self> {
        AddressDisplay { addr: self }
    }
}

/// Helper object for displaying an address
pub struct AddressDisplay<A> {
    addr: A,
}

impl<A: Address> fmt::Display for AddressDisplay<A> {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        match mem::size_of::<Self>() {
            1 => write!(f, "${:02X}", self.addr),
            2 => write!(f, "${:04X}", self.addr),
            4 => write!(f, "${:08X}", self.addr),
            _ => write!(f, "${:X}", self.addr),
        }
    }
}

// Supported address sizes
impl Address for u8 { }
impl Address for u16 { }
impl Address for u32 { }
impl Address for u64 { }

/// A trait for anything that has an address bus and can get/set data. The
/// data size that can be get/set is u8 always, the address size is given
/// as a type parameter and can be of any size (typically u16 or u32).
pub trait Addressable<A: Address> {
    /// Memory read: returns the data at the given address
    fn get (&self, addr: A) -> u8;

    /// Get a number in host platform byte order format from the given address.
    /// Note: Don't use this directly, better use `get_be` or `get_le` instead.
    fn get_number<T: Int> (&self, addr: A, mask: A) -> T {
        let mut val: T = Int::zero();
        let size = mem::size_of::<T>() as isize;
        let ptr = &mut val as *mut T as *mut u8;
        for i in 0..size {
            unsafe { *ptr.offset(i) = self.get(addr.offset_masked(i, mask)); }
        }
        val
    }

    /// Get a number in big endian format from the given address
    fn get_be<T: Int> (&self, addr: A) -> T {
        Int::from_be(self.get_number(addr, Int::zero()))
    }

    /// Get a number in big endian format from the given masked address
    fn get_be_masked<T: Int> (&self, addr: A, mask: A) -> T {
        Int::from_be(self.get_number(addr, mask))
    }

    /// Get a number in little endian format from the given address
    fn get_le<T: Int> (&self, addr: A) -> T {
        Int::from_le(self.get_number(addr, Int::zero()))
    }

    /// Get a number in little endian format from the given masked address
    fn get_le_masked<T: Int> (&self, addr: A, mask: A) -> T {
        Int::from_le(self.get_number(addr, mask))
    }

    /// Memory write: set the data at the given address
    fn set (&mut self, addr: A, data: u8);

    /// Store a number in host platform byte order format to the given address.
    /// Note: Don't use this directly, better use `set_be` or `set_le` instead.
    fn set_number<T: Int> (&mut self, addr: A, mask: A, val: T) {
        let size = mem::size_of::<T>() as isize;
        let ptr = &val as *const T as *const u8;
        for i in 0..size {
            unsafe { self.set(addr.offset_masked(i, mask), *ptr.offset(i)); }
        }
    }

    /// Store a number in big endian format to the given address
    fn set_be<T: Int> (&mut self, addr: A, val: T) {
        self.set_number(addr, Int::zero(), Int::to_be(val));
    }

    /// Store a number in big endian format to the given masked address
    fn set_be_masked<T: Int> (&mut self, addr: A, mask: A, val: T) {
        self.set_number(addr, mask, Int::to_be(val));
    }

    /// Store a number in little endian format to the given address
    fn set_le<T: Int> (&mut self, addr: A, val: T) {
        self.set_number(addr, Int::zero(), Int::to_le(val));
    }

    /// Store a number in little endian format to the given masked address
    fn set_le_masked<T: Int> (&mut self, addr: A, mask: A, val: T) {
        self.set_number(addr, mask, Int::to_le(val));
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
        for addr in self.addr1..self.addr2 {
            try!(write!(f, "{:02X} ", self.mem.get(addr)));
        }
        write!(f, "{:02X}", self.mem.get(self.addr2))
    }
}


#[cfg(test)]
mod test {
    use super::{Address, Addressable};

    #[test]
    fn address_offset () {
        assert_eq!(0x1234_u16.offset( 5), 0x1239_u16);
        assert_eq!(0x1234_u16.offset(-3), 0x1231_u16);
        assert_eq!(0xffff_u16.offset( 1), 0x0000_u16);
        assert_eq!(0x0000_u16.offset(-1), 0xffff_u16);
    }

    #[test]
    fn address_offset_masked () {
        assert_eq!(0x12ff_u16.offset_masked( 1, 0x0000), 0x1300_u16);
        assert_eq!(0x12ff_u16.offset_masked( 1, 0xff00), 0x1200_u16);
        assert_eq!(0x1300_u16.offset_masked(-1, 0x0000), 0x12ff_u16);
        assert_eq!(0x1300_u16.offset_masked(-1, 0xff00), 0x13ff_u16);
    }

    #[test]
    fn address_display () {
        assert_eq!(format!("{}", 0x0f_u8.display()), "$0F");
        assert_eq!(format!("{}", 0x01ff_u16.display()), "$01FF");
    }

    struct DummyData;
    impl Addressable<u16> for DummyData {
        fn get (&self, addr: u16) -> u8 { addr as u8 }
        fn set (&mut self, addr: u16, data: u8) { assert_eq!(data, addr as u8); }
    }

    #[test]
    fn get_byte () {
        let data = DummyData;
        assert_eq!(data.get(0x0012_u16), 0x12);
        assert_eq!(data.get(0x1234_u16), 0x34);
    }

    #[test]
    fn get_big_endian_number () {
        let data = DummyData;
        assert_eq!(      0x02_u8 , data.get_be(0x0002_u16));
        assert_eq!(      0x54_u8 , data.get_be(0x0054_u16));
        assert_eq!(    0x0203_u16, data.get_be(0x0002_u16));
        assert_eq!(    0x5455_u16, data.get_be(0x0054_u16));
        assert_eq!(0x02030405_u32, data.get_be(0x0002_u16));
        assert_eq!(0x54555657_u32, data.get_be(0x0054_u16));
    }

    #[test]
    fn get_signed_big_endian_number () {
        let data = DummyData;
        assert_eq!(       0x54_i8 , data.get_be(0x0054_u16));
        assert_eq!(      -0x5b_i8 , data.get_be(0x00a5_u16));
        assert_eq!(     0x5455_i16, data.get_be(0x0054_u16));
        assert_eq!(    -0x5a5a_i16, data.get_be(0x00a5_u16));
        assert_eq!( 0x54555657_i32, data.get_be(0x0054_u16));
        assert_eq!(-0x5a595858_i32, data.get_be(0x00a5_u16));
    }

    #[test]
    fn get_masked_big_endian_number () {
        let data = DummyData;
        assert_eq!(    0xff00_u16, data.get_be_masked(0x12ff_u16, 0xff00));
        assert_eq!(0xfeff0001_u32, data.get_be_masked(0x12fe_u16, 0xff00));
    }

    #[test]
    fn get_little_endian_number () {
        let data = DummyData;
        assert_eq!(      0x02_u8 , data.get_le(0x0002_u16));
        assert_eq!(      0x54_u8 , data.get_le(0x0054_u16));
        assert_eq!(    0x0302_u16, data.get_le(0x0002_u16));
        assert_eq!(    0x5554_u16, data.get_le(0x0054_u16));
        assert_eq!(0x05040302_u32, data.get_le(0x0002_u16));
        assert_eq!(0x57565554_u32, data.get_le(0x0054_u16));
    }

    #[test]
    fn get_signed_little_endian_number () {
        let data = DummyData;
        assert_eq!(       0x54_i8 , data.get_le(0x0054_u16));
        assert_eq!(      -0x5b_i8 , data.get_le(0x00a5_u16));
        assert_eq!(     0x5554_i16, data.get_le(0x0054_u16));
        assert_eq!(    -0x595b_i16, data.get_le(0x00a5_u16));
        assert_eq!( 0x57565554_i32, data.get_le(0x0054_u16));
        assert_eq!(-0x5758595b_i32, data.get_le(0x00a5_u16));
    }

    #[test]
    fn get_masked_little_endian_number () {
        let data = DummyData;
        assert_eq!(    0x00ff_u16, data.get_le_masked(0x12ff_u16, 0xff00));
        assert_eq!(0x0100fffe_u32, data.get_le_masked(0x12fe_u16, 0xff00));
    }

    #[test]
    fn set_byte () {
        let mut data = DummyData;
        data.set(0x0012_u16, 0x12);
        data.set(0x1234_u16, 0x34);
    }

    #[test]
    fn set_big_endian_number () {
        let mut data = DummyData;
        data.set_be(0x0002_u16,       0x02_u8 );
        data.set_be(0x0054_u16,       0x54_u8 );
        data.set_be(0x0002_u16,     0x0203_u16);
        data.set_be(0x0054_u16,     0x5455_u16);
        data.set_be(0x0002_u16, 0x02030405_u32);
        data.set_be(0x0054_u16, 0x54555657_u32);
    }

    #[test]
    fn set_signed_big_endian_number () {
        let mut data = DummyData;
        data.set_be(0x0054_u16,        0x54_i8 );
        data.set_be(0x00a5_u16,       -0x5b_i8 );
        data.set_be(0x0054_u16,      0x5455_i16);
        data.set_be(0x00a5_u16,     -0x5a5a_i16);
        data.set_be(0x0054_u16,  0x54555657_i32);
        data.set_be(0x00a5_u16, -0x5a595858_i32);
    }

    #[test]
    fn set_masked_big_endian_number () {
        let mut data = DummyData;
        data.set_be_masked(0x12ff_u16, 0xff00,     0xff00_u16);
        data.set_be_masked(0x12fe_u16, 0xff00, 0xfeff0001_u32);
    }

    #[test]
    fn set_little_endian_number () {
        let mut data = DummyData;
        data.set_le(0x0002_u16,       0x02_u8 );
        data.set_le(0x0054_u16,       0x54_u8 );
        data.set_le(0x0002_u16,     0x0302_u16);
        data.set_le(0x0054_u16,     0x5554_u16);
        data.set_le(0x0002_u16, 0x05040302_u32);
        data.set_le(0x0054_u16, 0x57565554_u32);
    }

    #[test]
    fn set_signed_little_endian_number () {
        let mut data = DummyData;
        data.set_le(0x0054_u16,        0x54_i8 );
        data.set_le(0x00a5_u16,       -0x5b_i8 );
        data.set_le(0x0054_u16,      0x5554_i16);
        data.set_le(0x00a5_u16,     -0x595b_i16);
        data.set_le(0x0054_u16,  0x57565554_i32);
        data.set_le(0x00a5_u16, -0x5758595b_i32);
    }

    #[test]
    fn set_masked_little_endian_number () {
        let mut data = DummyData;
        data.set_le_masked(0x12ff_u16, 0xff00,     0x00ff_u16);
        data.set_le_masked(0x12fe_u16, 0xff00, 0x0100fffe_u32);
    }

    #[test]
    fn copying_memory () {
        let data1 = DummyData;
        let mut data2 = DummyData;
        data2.copy(0x8000_u16, &data1, 0x0000_u16, 0x1000);
    }

    #[test]
    fn dumping_memory () {
        let data = DummyData;
        assert_eq!(format!("{}", data.hexdump(0x0100_u16, 0x0100_u16)), "00");
        assert_eq!(format!("{}", data.hexdump(0x0100_u16, 0x0101_u16)), "00 01");
        assert_eq!(format!("{}", data.hexdump(0x0100_u16, 0x0105_u16)), "00 01 02 03 04 05");
    }
}
