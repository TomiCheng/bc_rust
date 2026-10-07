//! Unit encoding of [`FixedBigUint`].

use super::FixedBigUint;
use crate::encoding::{self, Unit};
use crate::{ArrayEncoding, ConversionError, Limb, LimbArray, Word};

/// Decodes into `N` limbs; `InputTooLarge` when a bit would be lost.
fn decode<const N: usize, U: Unit>(
    len: usize,
    unit: impl Fn(usize) -> U,
) -> Result<FixedBigUint<N>, ConversionError> {
    let mut limbs = [Limb::new(0); N];
    encoding::decode_into(len, &unit, 0, &mut limbs);
    if !encoding::fits(&limbs, 0, len, &unit) {
        return Err(ConversionError::InputTooLarge);
    }
    Ok(FixedBigUint::new(LimbArray::new(limbs)))
}

macro_rules! impl_array_encoding {
    ($($unit:ty),*) => {$(
        /// Writes the full width of `N` limbs. Constant time, except that an
        /// input which does not fit is reported once all of it is read.
        impl<const N: usize> ArrayEncoding<$unit> for FixedBigUint<N> {
            type DecodeError = ConversionError;

            fn from_le(input: &[$unit]) -> Result<Self, ConversionError> {
                decode(input.len(), |index| input[index])
            }

            fn from_be(input: &[$unit]) -> Result<Self, ConversionError> {
                decode(input.len(), |index| input[input.len() - 1 - index])
            }

            fn write_le(&self, output: &mut [$unit]) -> Result<usize, ConversionError> {
                let length = <Self as ArrayEncoding<$unit>>::length(self);
                encoding::write_le(self.as_limbs(), 0, length, output)
            }

            fn write_be(&self, output: &mut [$unit]) -> Result<usize, ConversionError> {
                let length = <Self as ArrayEncoding<$unit>>::length(self);
                encoding::write_be(self.as_limbs(), 0, length, output)
            }

            fn length(&self) -> usize {
                encoding::units_for_bits::<$unit>(N * Word::BITS as usize)
            }
        }
    )*};
}

impl_array_encoding!(u8, u32, u64);

#[cfg(test)]
mod tests {
    use super::FixedBigUint;
    use crate::{ArrayEncoding, ConversionError, Limb, Word};

    const WORD_BYTES: usize = Word::BITS as usize / 8;

    #[test]
    fn a_short_input_fills_the_low_limbs_and_zeroes_the_rest() {
        let value = FixedBigUint::<2>::from_le(&[42u8]).unwrap();
        assert_eq!(value.as_limbs(), [Limb::new(42), Limb::new(0)]);
        let value = FixedBigUint::<2>::from_be(&[1u8, 0]).unwrap();
        assert_eq!(value.as_limbs(), FixedBigUint::<2>::from(256u16).as_limbs());
    }

    #[test]
    fn the_writer_emits_the_full_width_and_rejects_a_short_buffer() {
        let value = FixedBigUint::<2>::from(42u8);
        let mut output = [0xaau8; 32];
        assert_eq!(value.write_le(&mut output), Ok(2 * WORD_BYTES));
        assert_eq!(output[0], 42);
        assert!(output[1..2 * WORD_BYTES].iter().all(|&byte| byte == 0));
        assert_eq!(value.write_be(&mut output), Ok(2 * WORD_BYTES));
        assert_eq!(output[2 * WORD_BYTES - 1], 42);
        assert_eq!(ArrayEncoding::<u8>::length(&value), 2 * WORD_BYTES);
        let mut short = [0u8; 1];
        assert_eq!(
            value.write_le(&mut short),
            Err(ConversionError::BufferTooSmall)
        );
    }

    #[test]
    fn an_input_wider_than_n_limbs_fits_only_when_the_extra_bits_are_zero() {
        let mut buffer = [0u8; 16];
        let input = &mut buffer[..WORD_BYTES + 1];
        assert!(FixedBigUint::<1>::from_le(input).is_ok());
        input[WORD_BYTES] = 1;
        assert_eq!(
            FixedBigUint::<1>::from_le(input).err(),
            Some(ConversionError::InputTooLarge)
        );
    }

    #[test]
    fn units_wider_than_a_limb_are_split_and_checked() {
        // a u64 unit spans two limbs on a 32-bit target
        assert_eq!(
            FixedBigUint::<1>::from_le(&[u64::MAX]).is_ok(),
            Word::BITS == 64
        );
        let mut output = [0u64; 1];
        assert_eq!(FixedBigUint::<1>::from(7u8).write_le(&mut output), Ok(1));
        assert_eq!(output, [7]);
    }

    #[test]
    fn zero_limbs_decode_only_zero_and_write_nothing() {
        assert!(FixedBigUint::<0>::from_le(&[0u8]).is_ok());
        assert!(FixedBigUint::<0>::from_le(&[1u8]).is_err());
        let value = FixedBigUint::<0>::from_le(&[] as &[u8]).unwrap();
        assert_eq!(ArrayEncoding::<u8>::length(&value), 0);
    }
}
