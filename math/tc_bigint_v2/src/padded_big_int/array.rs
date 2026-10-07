//! Unit encoding of [`PaddedBigInt`].

use core::convert::Infallible;

use super::PaddedBigInt;
use crate::encoding;
use crate::{ArrayEncoding, ConversionError, Word};

macro_rules! impl_array_encoding {
    ($($unit:ty),*) => {$(
        /// Two's complement; takes the width of the input, sign-extended to
        /// whole limbs, and writes the full width. Constant time.
        impl ArrayEncoding<$unit> for PaddedBigInt {
            type DecodeError = Infallible;

            fn from_le(input: &[$unit]) -> Result<Self, Infallible> {
                let unit = |index| input[index];
                let fill = encoding::input_fill(input.len(), unit);
                let limbs = encoding::decode_vec(input.len(), unit, fill);
                Ok(Self::new(limbs.into_boxed_slice()))
            }

            fn from_be(input: &[$unit]) -> Result<Self, Infallible> {
                let unit = |index| input[input.len() - 1 - index];
                let fill = encoding::input_fill(input.len(), unit);
                let limbs = encoding::decode_vec(input.len(), unit, fill);
                Ok(Self::new(limbs.into_boxed_slice()))
            }

            fn write_le(&self, output: &mut [$unit]) -> Result<usize, ConversionError> {
                let length = <Self as ArrayEncoding<$unit>>::length(self);
                let fill = encoding::sign_fill(self.as_limbs());
                encoding::write_le(self.as_limbs(), fill, length, output)
            }

            fn write_be(&self, output: &mut [$unit]) -> Result<usize, ConversionError> {
                let length = <Self as ArrayEncoding<$unit>>::length(self);
                let fill = encoding::sign_fill(self.as_limbs());
                encoding::write_be(self.as_limbs(), fill, length, output)
            }

            fn length(&self) -> usize {
                encoding::units_for_bits::<$unit>(self.as_limbs().len() * Word::BITS as usize)
            }
        }
    )*};
}

impl_array_encoding!(u8, u32, u64);

#[cfg(test)]
mod tests {
    use super::PaddedBigInt;
    use crate::{ArrayEncoding, Limb, Word};

    const WORD_BYTES: usize = Word::BITS as usize / 8;

    #[test]
    fn a_negative_input_is_sign_extended_to_whole_limbs() {
        let value = PaddedBigInt::from_be(&[0xffu8]).unwrap();
        assert_eq!(value.as_limbs(), [Limb::new(Word::MAX)]);
        let bytes = ArrayEncoding::<u8>::to_be(&value);
        assert_eq!(bytes.len(), WORD_BYTES);
        assert!(bytes.iter().all(|&byte| byte == 0xff));
        assert_eq!(
            PaddedBigInt::from_le(&[0x7fu8]).unwrap().as_limbs(),
            [Limb::new(0x7f)]
        );
    }

    #[test]
    fn a_round_trip_keeps_the_value_and_the_width() {
        let value = PaddedBigInt::from(-2i128);
        let bytes = ArrayEncoding::<u8>::to_le(&value);
        assert_eq!(bytes, (-2i128).to_le_bytes());
        assert_eq!(
            PaddedBigInt::from_le(&bytes).unwrap().as_limbs(),
            value.as_limbs()
        );
    }
}
