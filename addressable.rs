use std::num;

pub trait Addressable<ADDR: Int, DATA: Int> {
	pub fn get (&self, addr: ADDR) -> DATA;
}

// FIXME: With default methods, we won't need this anymore
trait AddressableUtil<ADDR: Int, DATA: Int> {
	pub fn get_be<T: Int> (&self, addr: ADDR) -> T;
	pub fn get_le<T: Int> (&self, addr: ADDR) -> T;
}

impl<ADDR: Int+Copy, DATA: Int, A: Addressable<ADDR, DATA>> AddressableUtil<ADDR, DATA> for A {
	pub fn get_be<T: Int> (&self, addr: ADDR) -> T {
		assert!(num::Primitive::bits::<T>() % num::Primitive::bits::<DATA>() == 0);
		let count = num::Primitive::bits::<T>() / num::Primitive::bits::<DATA>();
		let mut val = num::Zero::zero::<T>();
		let mut i = count;
		while i > 0 {
			i -= 1;
			let d: T = num::cast(self.get(addr + num::cast(i)));
			let shift: T = num::cast((count-i-1) * num::Primitive::bits::<DATA>());
			val = val + (d << shift);
		}
		val
	}

	pub fn get_le<T: Int> (&self, addr: ADDR) -> T {
		assert!(num::Primitive::bits::<T>() % num::Primitive::bits::<DATA>() == 0);
		let count = num::Primitive::bits::<T>() / num::Primitive::bits::<DATA>();
		let mut val = num::Zero::zero::<T>();
		let mut i = count;
		while i > 0 {
			i -= 1;
			let d: T = num::cast(self.get(addr + num::cast(i)));
			let shift: T = num::cast(i * num::Primitive::bits::<DATA>());
			val = val + (d << shift);
		}
		val
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	struct DummyData;
	impl Addressable<u16, u8> for DummyData {
		pub fn get (&self, addr: u16) -> u8 {
			addr as u8
		}
	}

	#[test]
	fn test_get () {
		let data = DummyData;
		assert_eq!(data.get(123), 123);
		assert_eq!(data.get(1234), 210);
	}

	#[test]
	fn test_get_big_endian () {
		let data = DummyData;
		assert_eq!(         2u8 , data.get_be(  2u16));
		assert_eq!(       123u8 , data.get_be(123u16));
		assert_eq!(       515u16, data.get_be(  2u16));		//   2<<8 |   3
		assert_eq!(     31612u16, data.get_be(123u16));		// 123<<8 | 124
		assert_eq!(  33752069u32, data.get_be(  2u16));		//   2<<24 |   3<<16 |   4<<8 |   5
		assert_eq!(2071756158u32, data.get_be(123u16));		// 123<<24 | 124<<16 | 125<<8 | 126
	}

	#[test]
	fn test_get_little_endian () {
		let data = DummyData;
		assert_eq!(         2u8 , data.get_le(  2u16));
		assert_eq!(       123u8 , data.get_le(123u16));
		assert_eq!(       770u16, data.get_le(  2u16));		//   2 |   3<<8
		assert_eq!(     31867u16, data.get_le(123u16));		// 123 | 124<<8
		assert_eq!(  84148994u32, data.get_le(  2u16));		//   2 |   3<<8 |   4<<16 |   5<<24
		assert_eq!(2122153083u32, data.get_le(123u16));		// 123 | 124<<8 | 125<<16 | 126<<24
	}
}
