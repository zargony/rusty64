use std::{fmt, num, vec};

/// A trait for all addresses
pub trait Addr: Int + Unsigned + fmt::UpperHex {
	/// Calculate new address with given offset
	fn offset<T: Integer+Signed+NumCast> (&self, offset: T) -> Self {
		if offset.is_negative() {
			self - num::cast(offset.abs()).unwrap()
		} else {
			self + num::cast(offset).unwrap()
		}
	}

	/// Calculate new address with given offset only changing the masked part of the address
	fn offset_masked<T: Integer+Signed+Bitwise+NumCast> (&self, offset: T, mask: Self) -> Self {
		(self & !mask) | (self.offset(offset) & mask)
	}
}

// Supported address sizes
// TODO: Shall we support arbitrary address sizes like u12?
impl Addr for u8 { }
impl Addr for u16 { }
impl Addr for u32 { }
impl Addr for u64 { }

/// Create a number from the given byte iterator in big endian order
/// FIXME: This should actually take an Iterator<&'a u8>
fn number_from_bytes<I: Iterator<u8>, T: Primitive> (mut it: I) -> T {
	let val = it.fold(0u64, |v, b| (v << 8) + b as u64);
	if Primitive::is_signed(None::<T>) {
		let shift = 64 - Primitive::bits(None::<T>);
		num::cast((val << shift) as i64 >> shift).unwrap()
	} else {
		num::cast(val).unwrap()
	}
}

/// Create a number from the given bytes in big endian order
fn number_from_be_bytes<T: Primitive> (data: &[u8]) -> T {
	number_from_bytes(data.iter().map(|b| *b))
}

/// Create a number from the given bytes in little endian order
fn number_from_le_bytes<T: Primitive> (data: &[u8]) -> T {
	number_from_bytes(data.rev_iter().map(|b| *b))
}

/// Convert a given number to bytes in big endian order
fn number_to_be_bytes<T: Primitive, U> (val: T, f: |&[u8]| -> U) -> U {
	let size = Primitive::bytes(None::<T>);
	let ptr = &val as *T as *u8;
	f(vec::from_fn(size, |i| unsafe { *ptr.offset((size - i - 1) as int) }))
}

/// Convert a given number to bytes in little endian order
fn number_to_le_bytes<T: Primitive, U> (val: T, f: |&[u8]| -> U) -> U {
	let size = Primitive::bytes(None::<T>);
	let ptr = &val as *T as *u8;
	unsafe { vec::raw::buf_as_slice(ptr, size, f) }
}

/// A trait for anything that has an address bus and can get/set data. The
/// data size that can be get/set is u8 always, the address size is given
/// as a type parameter and can be of any size (typically u16 or u32).
/// TODO: Support data sizes other than u8?
pub trait Addressable<A: Addr> {
	/// Memory read: returns the data at the given address
	fn get (&self, addr: A) -> u8;

	/// Get a number in big endian format from the given address
	fn get_be<T: Primitive> (&self, addr: A) -> T {
		let data = vec::from_fn(Primitive::bytes(None::<T>), |i| self.get(addr.offset(i as int)));
		number_from_be_bytes(data)
	}

	/// Get a number in big endian format from the given masked address
	fn get_be_masked<T: Primitive> (&self, addr: A, mask: A) -> T {
		let data = vec::from_fn(Primitive::bytes(None::<T>), |i| self.get(addr.offset_masked(i as int, mask.clone())));
		number_from_be_bytes(data)
	}

	/// Get a number in little endian format from the given address
	fn get_le<T: Primitive> (&self, addr: A) -> T {
		let data = vec::from_fn(Primitive::bytes(None::<T>), |i| self.get(addr.offset(i as int)));
		number_from_le_bytes(data)
	}

	/// Get a number in little endian format from the given masked address
	fn get_le_masked<T: Primitive> (&self, addr: A, mask: A) -> T {
		let data = vec::from_fn(Primitive::bytes(None::<T>), |i| self.get(addr.offset_masked(i as int, mask.clone())));
		number_from_le_bytes(data)
	}

	/// Memory write: set the data at the given address
	fn set (&mut self, addr: A, data: u8);

	/// Store a number in big endian format to the given address
	fn set_be<T: Primitive> (&mut self, addr: A, val: T) {
		number_to_be_bytes(val, |data| {
			for (i, &b) in data.iter().enumerate() {
				self.set(addr.offset(i as int), b);
			}
		});
	}

	/// Store a number in big endian format to the given masked address
	fn set_be_masked<T: Primitive> (&mut self, addr: A, mask: A, val: T) {
		number_to_be_bytes(val, |data| {
			for (i, &b) in data.iter().enumerate() {
				self.set(addr.offset_masked(i as int, mask.clone()), b);
			}
		});
	}

	/// Store a number in little endian format to the given address
	fn set_le<T: Primitive> (&mut self, addr: A, val: T) {
		number_to_le_bytes(val, |data| {
			for (i, &b) in data.iter().enumerate() {
				self.set(addr.offset(i as int), b);
			}
		});
	}

	/// Store a number in little endian format to the given masked address
	fn set_le_masked<T: Primitive> (&mut self, addr: A, mask: A, val: T) {
		number_to_le_bytes(val, |data| {
			for (i, &b) in data.iter().enumerate() {
				self.set(addr.offset_masked(i as int, mask.clone()), b);
			}
		});
	}
}


#[cfg(test)]
mod test {
	use super::{number_from_be_bytes, number_from_le_bytes};
	use super::{number_to_be_bytes, number_to_le_bytes};
	use super::{Addr, Addressable};

	#[test]
	fn address_offset () {
		assert_eq!(0x1234_u16.offset( 5), 0x1239_u16);
		assert_eq!(0x1234_u16.offset(-3), 0x1231_u16);
		assert_eq!(0xffff_u16.offset( 1), 0x0000_u16);
		assert_eq!(0x0000_u16.offset(-1), 0xffff_u16);
	}

	#[test]
	fn address_offset_masked () {
		assert_eq!(0x12ff_u16.offset_masked( 1, 0xffff), 0x1300_u16);
		assert_eq!(0x12ff_u16.offset_masked( 1, 0x00ff), 0x1200_u16);
		assert_eq!(0x1300_u16.offset_masked(-1, 0xffff), 0x12ff_u16);
		assert_eq!(0x1300_u16.offset_masked(-1, 0x00ff), 0x13ff_u16);
	}

	#[test]
	fn convert_from_big_endian () {
		assert_eq!(       0x12_u8 , number_from_be_bytes(&[0x12]));
		assert_eq!(       0x98_u8 , number_from_be_bytes(&[0x98]));
		assert_eq!( 0x12345678_u32, number_from_be_bytes(&[0x12, 0x34, 0x56, 0x78]));
		assert_eq!( 0x98765432_u32, number_from_be_bytes(&[0x98, 0x76, 0x54, 0x32]));
		assert_eq!(       0x12_i8 , number_from_be_bytes(&[0x12]));
		assert_eq!(      -0x68_i8 , number_from_be_bytes(&[0x98]));
		assert_eq!( 0x12345678_i32, number_from_be_bytes(&[0x12, 0x34, 0x56, 0x78]));
		assert_eq!(-0x6789abce_i32, number_from_be_bytes(&[0x98, 0x76, 0x54, 0x32]));
		assert_eq!(     0x9876_u32, number_from_be_bytes(&[0x98, 0x76]));
		assert_eq!(     0x9876_i32, number_from_be_bytes(&[0x98, 0x76]));
	}

	#[test]
	fn convert_from_little_endian () {
		assert_eq!(       0x12_u8 , number_from_le_bytes(&[0x12]));
		assert_eq!(       0x98_u8 , number_from_le_bytes(&[0x98]));
		assert_eq!( 0x12345678_u32, number_from_le_bytes(&[0x78, 0x56, 0x34, 0x12]));
		assert_eq!( 0x98765432_u32, number_from_le_bytes(&[0x32, 0x54, 0x76, 0x98]));
		assert_eq!(       0x12_i8 , number_from_le_bytes(&[0x12]));
		assert_eq!(      -0x68_i8 , number_from_le_bytes(&[0x98]));
		assert_eq!( 0x12345678_i32, number_from_le_bytes(&[0x78, 0x56, 0x34, 0x12]));
		assert_eq!(-0x6789abce_i32, number_from_le_bytes(&[0x32, 0x54, 0x76, 0x98]));
		assert_eq!(     0x9876_u32, number_from_le_bytes(&[0x76, 0x98]));
		assert_eq!(     0x9876_i32, number_from_le_bytes(&[0x76, 0x98]));
	}

	#[test]
	fn convert_to_big_endian () {
		number_to_be_bytes(       0x12_u8 , |data| assert_eq!(data, [0x12]));
		number_to_be_bytes(       0x98_u8 , |data| assert_eq!(data, [0x98]));
		number_to_be_bytes( 0x12345678_u32, |data| assert_eq!(data, [0x12, 0x34, 0x56, 0x78]));
		number_to_be_bytes( 0x98765432_u32, |data| assert_eq!(data, [0x98, 0x76, 0x54, 0x32]));
		number_to_be_bytes(       0x12_i8 , |data| assert_eq!(data, [0x12]));
		number_to_be_bytes(      -0x68_i8 , |data| assert_eq!(data, [0x98]));
		number_to_be_bytes( 0x12345678_i32, |data| assert_eq!(data, [0x12, 0x34, 0x56, 0x78]));
		number_to_be_bytes(-0x6789abce_i32, |data| assert_eq!(data, [0x98, 0x76, 0x54, 0x32]));
		number_to_be_bytes(     0x9876_u32, |data| assert_eq!(data, [0x00, 0x00, 0x98, 0x76]));
		number_to_be_bytes(     0x9876_i32, |data| assert_eq!(data, [0x00, 0x00, 0x98, 0x76]));
	}

	#[test]
	fn convert_to_little_endian () {
		number_to_le_bytes(       0x12_u8 , |data| assert_eq!(data, [0x12]));
		number_to_le_bytes(       0x98_u8 , |data| assert_eq!(data, [0x98]));
		number_to_le_bytes( 0x12345678_u32, |data| assert_eq!(data, [0x78, 0x56, 0x34, 0x12]));
		number_to_le_bytes( 0x98765432_u32, |data| assert_eq!(data, [0x32, 0x54, 0x76, 0x98]));
		number_to_le_bytes(       0x12_i8 , |data| assert_eq!(data, [0x12]));
		number_to_le_bytes(      -0x68_i8 , |data| assert_eq!(data, [0x98]));
		number_to_le_bytes( 0x12345678_i32, |data| assert_eq!(data, [0x78, 0x56, 0x34, 0x12]));
		number_to_le_bytes(-0x6789abce_i32, |data| assert_eq!(data, [0x32, 0x54, 0x76, 0x98]));
		number_to_le_bytes(     0x9876_u32, |data| assert_eq!(data, [0x76, 0x98, 0x00, 0x00]));
		number_to_le_bytes(     0x9876_i32, |data| assert_eq!(data, [0x76, 0x98, 0x00, 0x00]));
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
		assert_eq!(    0xff00_u16, data.get_be_masked(0x12ff_u16, 0x00ff));
		assert_eq!(0xfeff0001_u32, data.get_be_masked(0x12fe_u16, 0x00ff));
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
		assert_eq!(    0x00ff_u16, data.get_le_masked(0x12ff_u16, 0x00ff));
		assert_eq!(0x0100fffe_u32, data.get_le_masked(0x12fe_u16, 0x00ff));
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
		data.set_be_masked(0x12ff_u16, 0x00ff,     0xff00_u16);
		data.set_be_masked(0x12fe_u16, 0x00ff, 0xfeff0001_u32);
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
		data.set_le_masked(0x12ff_u16, 0x00ff,     0x00ff_u16);
		data.set_le_masked(0x12fe_u16, 0x00ff, 0x0100fffe_u32);
	}
}
