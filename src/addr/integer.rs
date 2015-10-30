//!
//! Generic integer
//!

/// Trait for integers that can be converted from host byte order to a fixed byte order and
/// vice versa (all Rust built-in integers). This used to be in std::num or in the num crate,
/// but has become deprecated for some reason. Rust still provides the conversion methods, but
/// is missing this trait that groups all convertable integer types.
pub trait Integer: Copy {
    fn from_be (x: Self) -> Self;
    fn from_le (x: Self) -> Self;
    fn to_be (self) -> Self;
    fn to_le (self) -> Self;
}

macro_rules! impl_integer {
    ($($T:ty)*) => ($(
        impl Integer for $T {
            fn from_be (x: $T) -> $T { <$T>::from_be(x) }
            fn from_le (x: $T) -> $T { <$T>::from_le(x) }
            fn to_be (self) -> $T { <$T>::to_be(self) }
            fn to_le (self) -> $T { <$T>::to_le(self) }
        }
    )*);
}

impl_integer!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize);
