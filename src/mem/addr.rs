use std::{num, vec};

/// A trait for all addresses
pub trait Addr: Int + Unsigned {
	/// Calculate new address with given offset from given address
	fn offset<T: Integer+Signed+NumCast> (&self, offset: T) -> Self {
		if offset.is_negative() {
			self - num::cast(offset.abs()).unwrap()
		} else {
			self + num::cast(offset).unwrap()
		}
	}
}

/// Supported address sizes
impl Addr for u8 { }
impl Addr for u16 { }
impl Addr for u32 { }
impl Addr for u64 { }
// TODO: Shall we support arbitrary address sizes like u12?

/// Create a number from the given byte iterator in big endian order
/// FIXME: This should actually take an Iterator<&'a u8>
fn number_from_bytes<I: Iterator<u8>, T: Primitive> (mut it: I) -> T {
	let val = it.fold(0u64, |v, b| (v << 8) + b as u64);
	if num::Primitive::is_signed(None::<T>) {
		let shift = 64 - num::Primitive::bits(None::<T>);
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
	let size = num::Primitive::bytes(None::<T>);
	let ptr = &val as *T as *u8;
	f(vec::from_fn(size, |i| unsafe { *ptr.offset((size - i - 1) as int) }))
}

/// Convert a given number to bytes in little endian order
fn number_to_le_bytes<T: Primitive, U> (val: T, f: |&[u8]| -> U) -> U {
	let size = num::Primitive::bytes(None::<T>);
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

	/// Returns the data at the given address with given offset
	fn getx (&self, addr: A, ofs: int) -> u8 {
		self.get(addr.offset(ofs))
	}

	/// Get a number in big endian format from the given address
	fn get_be<T: Primitive> (&self, addr: A) -> T {
		let data = vec::from_fn(num::Primitive::bytes(None::<T>), |i| self.getx(addr.clone(), i as int));
		number_from_be_bytes(data)
	}

	/// Get a number in little endian format from the given address
	fn get_le<T: Primitive> (&self, addr: A) -> T {
		let data = vec::from_fn(num::Primitive::bytes(None::<T>), |i| self.getx(addr.clone(), i as int));
		number_from_le_bytes(data)
	}

	/// Memory write: set the data at the given address
	fn set (&mut self, addr: A, data: u8);

	/// Set the data at the given address with given offset
	fn setx (&mut self, addr: A, ofs: int, data: u8) {
		self.set(addr.offset(ofs), data);
	}

	/// Store a number in big endian format to the given address
	fn set_be<T: Primitive> (&mut self, addr: A, val: T) {
		number_to_be_bytes(val, |data| {
			for (i, &b) in data.iter().enumerate() {
				self.setx(addr.clone(), i as int, b);
			}
		});
	}

	/// Store a number in little endian format to the given address
	fn set_le<T: Primitive> (&mut self, addr: A, val: T) {
		number_to_le_bytes(val, |data| {
			for (i, &b) in data.iter().enumerate() {
				self.setx(addr.clone(), i as int, b);
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
	fn test_offset () {
		assert_eq!(0x1234_u16.offset( 5), 0x1239_u16);
		assert_eq!(0x1234_u16.offset(-3), 0x1231_u16);
		assert_eq!(0xffff_u16.offset( 1), 0x0000_u16);
		assert_eq!(0x0000_u16.offset(-1), 0xffff_u16);
	}

	#[test]
	fn test_number_from_be_bytes () {
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
	fn test_number_from_le_bytes () {
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
	fn test_number_to_be_bytes () {
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
	fn test_number_to_le_bytes () {
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
		fn get (&self, addr: u16) -> u8 {
			addr as u8
		}
		fn set (&mut self, addr: u16, data: u8) {
			assert_eq!(data, addr as u8);
		}
	}

	#[test]
	fn test_get () {
		let data = DummyData;
		assert_eq!(data.get(0x0012_u16), 0x12);
		assert_eq!(data.get(0x1234_u16), 0x34);
	}

	#[test]
	fn test_getx () {
		let data = DummyData;
		assert_eq!(data.getx(0x1234_u16,  5), 0x39);
		assert_eq!(data.getx(0x1234_u16, -3), 0x31);
		assert_eq!(data.getx(0xffff_u16,  1), 0x00);
		assert_eq!(data.getx(0x0000_u16, -1), 0xff);
	}

	#[test]
	fn test_get_big_endian () {
		let data = DummyData;
		assert_eq!(      0x02_u8 , data.get_be(0x0002_u16));
		assert_eq!(      0x54_u8 , data.get_be(0x0054_u16));
		assert_eq!(    0x0203_u16, data.get_be(0x0002_u16));
		assert_eq!(    0x5455_u16, data.get_be(0x0054_u16));
		assert_eq!(0x02030405_u32, data.get_be(0x0002_u16));
		assert_eq!(0x54555657_u32, data.get_be(0x0054_u16));
	}

	#[test]
	fn test_get_big_endian_signed () {
		let data = DummyData;
		assert_eq!(       0x54_i8 , data.get_be(0x0054_u16));
		assert_eq!(      -0x5b_i8 , data.get_be(0x00a5_u16));
		assert_eq!(     0x5455_i16, data.get_be(0x0054_u16));
		assert_eq!(    -0x5a5a_i16, data.get_be(0x00a5_u16));
		assert_eq!( 0x54555657_i32, data.get_be(0x0054_u16));
		assert_eq!(-0x5a595858_i32, data.get_be(0x00a5_u16));
	}

	#[test]
	fn test_get_little_endian () {
		let data = DummyData;
		assert_eq!(      0x02_u8 , data.get_le(0x0002_u16));
		assert_eq!(      0x54_u8 , data.get_le(0x0054_u16));
		assert_eq!(    0x0302_u16, data.get_le(0x0002_u16));
		assert_eq!(    0x5554_u16, data.get_le(0x0054_u16));
		assert_eq!(0x05040302_u32, data.get_le(0x0002_u16));
		assert_eq!(0x57565554_u32, data.get_le(0x0054_u16));
	}

	#[test]
	fn test_get_little_endian_signed () {
		let data = DummyData;
		assert_eq!(       0x54_i8 , data.get_le(0x0054_u16));
		assert_eq!(      -0x5b_i8 , data.get_le(0x00a5_u16));
		assert_eq!(     0x5554_i16, data.get_le(0x0054_u16));
		assert_eq!(    -0x595b_i16, data.get_le(0x00a5_u16));
		assert_eq!( 0x57565554_i32, data.get_le(0x0054_u16));
		assert_eq!(-0x5758595b_i32, data.get_le(0x00a5_u16));
	}

	#[test]
	fn test_set () {
		let mut data = DummyData;
		data.set(0x0012_u16, 0x12);
		data.set(0x1234_u16, 0x34);
	}

	#[test]
	fn test_setx () {
		let mut data = DummyData;
		data.setx(0x1234_u16,  5, 0x39);
		data.setx(0x1234_u16, -3, 0x31);
		data.setx(0xffff_u16,  1, 0x00);
		data.setx(0x0000_u16, -1, 0xff);
	}

	#[test]
	fn test_set_big_endian () {
		let mut data = DummyData;
		data.set_be(0x0002_u16,       0x02_u8 );
		data.set_be(0x0054_u16,       0x54_u8 );
		data.set_be(0x0002_u16,     0x0203_u16);
		data.set_be(0x0054_u16,     0x5455_u16);
		data.set_be(0x0002_u16, 0x02030405_u32);
		data.set_be(0x0054_u16, 0x54555657_u32);
	}

	#[test]
	fn test_set_big_endian_signed () {
		let mut data = DummyData;
		data.set_be(0x0054_u16,        0x54_i8 );
		data.set_be(0x00a5_u16,       -0x5b_i8 );
		data.set_be(0x0054_u16,      0x5455_i16);
		data.set_be(0x00a5_u16,     -0x5a5a_i16);
		data.set_be(0x0054_u16,  0x54555657_i32);
		data.set_be(0x00a5_u16, -0x5a595858_i32);
	}

	#[test]
	fn test_set_little_endian () {
		let mut data = DummyData;
		data.set_le(0x0002_u16,       0x02_u8 );
		data.set_le(0x0054_u16,       0x54_u8 );
		data.set_le(0x0002_u16,     0x0302_u16);
		data.set_le(0x0054_u16,     0x5554_u16);
		data.set_le(0x0002_u16, 0x05040302_u32);
		data.set_le(0x0054_u16, 0x57565554_u32);
	}

	#[test]
	fn test_set_little_endian_signed () {
		let mut data = DummyData;
		data.set_le(0x0054_u16,        0x54_i8 );
		data.set_le(0x00a5_u16,       -0x5b_i8 );
		data.set_le(0x0054_u16,      0x5554_i16);
		data.set_le(0x00a5_u16,     -0x595b_i16);
		data.set_le(0x0054_u16,  0x57565554_i32);
		data.set_le(0x00a5_u16, -0x5758595b_i32);
	}
}
