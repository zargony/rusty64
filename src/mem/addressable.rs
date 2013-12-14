use std::num;

pub trait Addr: Int { }

impl Addr for u8 { }
impl Addr for u16 { }
impl Addr for u32 { }
impl Addr for u64 { }

pub trait Addressable<A: Addr> {
	fn get (&self, addr: A) -> u8;
	fn set (&mut self, addr: A, data: u8);

	fn getx (&self, addr:A, offset: int) -> u8 {
		if offset < 0 {
			self.get(addr - num::cast(-offset).unwrap())
		} else {
			self.get(addr + num::cast(offset).unwrap())
		}
	}

	fn get_be_n (&self, addr: A, nbytes: uint) -> u64 {
		assert!(nbytes > 0 && nbytes < 8);
		let mut val = 0u64;
		let mut i = nbytes;
		while i > 0 {
			i -= 1;
			val |= self.get(addr + num::cast(i).unwrap()) as u64 << (nbytes-i-1) * 8;
		}
		val
	}

	fn get_be<T: Int> (&self, addr: A) -> T {
		let none: Option<T> = None;
		num::cast(self.get_be_n(addr, num::Primitive::bytes(none))).unwrap()
	}

	fn get_le_n (&self, addr: A, nbytes: uint) -> u64 {
		assert!(nbytes > 0 && nbytes < 8);
		let mut val = 0u64;
		let mut i = nbytes;
		while i > 0 {
			i -= 1;
			val |= self.get(addr + num::cast(i).unwrap()) as u64 << i * 8;
		}
		val
	}

	fn get_le<T: Int> (&self, addr: A) -> T {
		let none: Option<T> = None;
		num::cast(self.get_le_n(addr, num::Primitive::bytes(none))).unwrap()
	}

	fn setx (&mut self, addr: A, offset: int, data: u8) {
		if offset < 0 {
			self.set(addr - num::cast(-offset).unwrap(), data);
		} else {
			self.set(addr + num::cast(offset).unwrap(), data);
		}
	}

	fn set_be_n (&mut self, addr: A, nbytes: uint, val: u64) {
		assert!(nbytes > 0 && nbytes < 8);
		let mut i = nbytes;
		while i > 0 {
			i -= 1;
			self.set(addr + num::cast(i).unwrap(), (val >> (nbytes-i-1) * 8) as u8);
		}
	}

	fn set_be<T: Int> (&mut self, addr: A, val: T) {
		let none: Option<T> = None;
		self.set_be_n(addr, num::Primitive::bytes(none), num::cast(val).unwrap());
	}

	fn set_le_n (&mut self, addr: A, nbytes: uint, val: u64) {
		assert!(nbytes > 0 && nbytes < 8);
		let mut i = nbytes;
		while i > 0 {
			i -= 1;
			self.set(addr + num::cast(i).unwrap(), (val >> i * 8) as u8);
		}
	}

	fn set_le<T: Int> (&mut self, addr: A, val: T) {
		let none: Option<T> = None;
		self.set_le_n(addr, num::Primitive::bytes(none), num::cast(val).unwrap());
	}
}


#[cfg(test)]
mod test {
	use super::Addressable;

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
		assert_eq!(data.get(0x0012), 0x12);
		assert_eq!(data.get(0x1234), 0x34);
	}

	#[test]
	fn test_getx () {
		let data = DummyData;
		assert_eq!(data.getx(0x0012_u16, 5), 0x17);
		assert_eq!(data.getx(0x1234_u16, 5), 0x39);
		assert_eq!(data.getx(0x0012_u16, -3), 0x0f);
		assert_eq!(data.getx(0x1234_u16, -3), 0x31);
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
	fn test_get_signed_big_endian () {
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
	fn test_get_signed_little_endian () {
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
		data.set(0x0012, 0x12);
		data.set(0x1234, 0x34);
	}

	#[test]
	fn test_setx () {
		let mut data = DummyData;
		data.setx(0x0012_u16, 5, 0x17);
		data.setx(0x1234_u16, 5, 0x39);
		data.setx(0x0012_u16, -3, 0x0f);
		data.setx(0x1234_u16, -3, 0x31);
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
	fn test_set_signed_big_endian () {
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
	fn test_set_signed_little_endian () {
		let mut data = DummyData;
		data.set_le(0x0054_u16,        0x54_i8 );
		data.set_le(0x00a5_u16,       -0x5b_i8 );
		data.set_le(0x0054_u16,      0x5554_i16);
		data.set_le(0x00a5_u16,     -0x595b_i16);
		data.set_le(0x0054_u16,  0x57565554_i32);
		data.set_le(0x00a5_u16, -0x5758595b_i32);
	}
}
