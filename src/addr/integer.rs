//! Generic integer

/// Trait for integers that can be converted from host byte order to a fixed byte order and
/// vice versa (all Rust built-in integers).
pub trait Integer<const N: usize>:
    num_traits::FromBytes<Bytes = [u8; N]> + num_traits::ToBytes<Bytes = [u8; N]>
{
}

impl<T, const N: usize> Integer<N> for T where
    T: num_traits::FromBytes<Bytes = [u8; N]> + num_traits::ToBytes<Bytes = [u8; N]>
{
}
