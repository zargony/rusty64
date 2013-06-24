use std::num;

pub trait Addressable<ADDR: Int, DATA: Int> {
	pub fn get (&self, addr: ADDR) -> DATA;
	pub fn set (&mut self, addr: ADDR, data: DATA);
}

// FIXME: With default methods, we won't need this anymore
pub trait AddressableUtil<ADDR: Int, DATA: Int> {
	pub fn get_be<T: Int> (&self, addr: ADDR) -> T;
	pub fn get_le<T: Int> (&self, addr: ADDR) -> T;
	pub fn set_be<T: Int> (&mut self, addr: ADDR, val: T);
	pub fn set_le<T: Int> (&mut self, addr: ADDR, val: T);
}

impl<ADDR: Int, DATA: Int, A: Addressable<ADDR, DATA>> AddressableUtil<ADDR, DATA> for A {
	pub fn get_be<T: Int> (&self, addr: ADDR) -> T {
		assert!(num::Primitive::bits::<T>() % num::Primitive::bits::<DATA>() == 0);
		let count = num::Primitive::bits::<T>() / num::Primitive::bits::<DATA>();
		let mut val = num::Zero::zero::<T>();
		let mut i = count;
		while i > 0 {
			i -= 1;
			let shift: T = num::cast((count-i-1) * num::Primitive::bits::<DATA>());
			let d: T = num::cast(self.get(addr + num::cast(i)));
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
			let shift: T = num::cast(i * num::Primitive::bits::<DATA>());
			let d: T = num::cast(self.get(addr + num::cast(i)));
			val = val + (d << shift);
		}
		val
	}

	pub fn set_be<T: Int> (&mut self, addr: ADDR, val: T) {
		assert!(num::Primitive::bits::<T>() % num::Primitive::bits::<DATA>() == 0);
		let count = num::Primitive::bits::<T>() / num::Primitive::bits::<DATA>();
		let mask = (1 << num::Primitive::bits::<DATA>()) - 1;
		let mut i = count;
		while i > 0 {
			i -= 1;
			let shift: T = num::cast((count-i-1) * num::Primitive::bits::<DATA>());
			let d: DATA = num::cast((val >> shift) & num::cast(mask));
			self.set(addr + num::cast(i), d);
		}
	}

	pub fn set_le<T: Int> (&mut self, addr: ADDR, val: T) {
		assert!(num::Primitive::bits::<T>() % num::Primitive::bits::<DATA>() == 0);
		let count = num::Primitive::bits::<T>() / num::Primitive::bits::<DATA>();
		let mask = (1 << num::Primitive::bits::<DATA>()) - 1;
		let mut i = count;
		while i > 0 {
			i -= 1;
			let shift: T = num::cast(i * num::Primitive::bits::<DATA>());
			let d: DATA = num::cast((val >> shift) & num::cast(mask));
			self.set(addr + num::cast(i), d);
		}
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
		pub fn set (&mut self, addr: u16, data: u8) {
			assert_eq!(data, addr as u8);
		}
	}

	#[test]
	fn test_get () {
		let data = DummyData;
		assert_eq!(data.get(0x12), 0x12);
		assert_eq!(data.get(0x1234), 0x34);
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
	fn test_set () {
		let mut data = DummyData;
		data.set(0x12, 0x12);
		data.set(0x1234, 0x34);
	}

	#[test]
	fn test_set_big_endian () {
		let mut data = DummyData;
		data.set_be(0x0002_u16, 0x02_u8);
		data.set_be(0x0054_u16, 0x54_u8);
		data.set_be(0x0002_u16, 0x0203_u16);
		data.set_be(0x0054_u16, 0x5455_u16);
		data.set_be(0x0002_u16, 0x02030405_u32);
		data.set_be(0x0054_u16, 0x54555657_u32);
	}

	#[test]
	fn test_set_little_endian () {
		let mut data = DummyData;
		data.set_le(0x0002_u16, 0x02_u8);
		data.set_le(0x0054_u16, 0x54_u8);
		data.set_le(0x0002_u16, 0x0302_u16);
		data.set_le(0x0054_u16, 0x5554_u16);
		data.set_le(0x0002_u16, 0x05040302_u32);
		data.set_le(0x0054_u16, 0x57565554_u32);
	}
}
