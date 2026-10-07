//! Array conversion contracts.

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

use crate::ConversionError;

/// Conversion between integers and slices of `T`, such as `u8`, `u32` or `u64`.
///
/// Signed implementations decode and write two's complement. Fixed-width
/// implementations report inputs which do not fit through
/// [`Self::DecodeError`] and write their full width; the others write the
/// shortest form. For a magnitude, take the unsigned absolute value first,
/// as [`FixedBigInt::unsigned_abs`](crate::FixedBigInt::unsigned_abs) gives.
/// Each implementation states whether it is constant time.
pub trait ArrayEncoding<T: Copy + Default>: Sized {
    /// Error returned when an input cannot be represented by `Self`.
    type DecodeError;

    /// Decodes little-endian units.
    fn from_le(input: &[T]) -> Result<Self, Self::DecodeError>;

    /// Decodes big-endian units.
    fn from_be(input: &[T]) -> Result<Self, Self::DecodeError>;

    /// Writes little-endian units into caller-provided storage, returning how
    /// many were written.
    fn write_le(&self, output: &mut [T]) -> Result<usize, ConversionError>;

    /// Writes big-endian units into caller-provided storage, returning how
    /// many were written.
    fn write_be(&self, output: &mut [T]) -> Result<usize, ConversionError>;

    /// Exact output length of [`Self::write_le`] and [`Self::write_be`].
    fn length(&self) -> usize;

    /// Allocates and returns little-endian units.
    #[cfg(feature = "alloc")]
    fn to_le(&self) -> Vec<T> {
        let mut output = vec![T::default(); self.length()];
        self.write_le(&mut output)
            .expect("output has the exact length");
        output
    }

    /// Allocates and returns big-endian units.
    #[cfg(feature = "alloc")]
    fn to_be(&self) -> Vec<T> {
        let mut output = vec![T::default(); self.length()];
        self.write_be(&mut output)
            .expect("output has the exact length");
        output
    }
}
