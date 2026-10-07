//! Unit encoding of [`FixedBigInt`].

use super::FixedBigInt;
use crate::encoding::{self, Unit};
use crate::{ArrayEncoding, ConversionError, Limb, LimbArray, Word};

/// Decodes two's complement into `N` limbs; `InputTooLarge` when the value
/// does not fit.
fn decode<const N: usize, U: Unit>(
    len: usize,
    unit: impl Fn(usize) -> U,
) -> Result<FixedBigInt<N>, ConversionError> {
    let mut limbs = [Limb::new(0); N];
    encoding::decode_into(len, &unit, encoding::input_fill(len, &unit), &mut limbs);
    // the bits beyond N limbs must repeat the sign the limbs ended up with
    if !encoding::fits(&limbs, encoding::sign_fill(&limbs), len, &unit) {
        return Err(ConversionError::InputTooLarge);
    }
    Ok(FixedBigInt::new(LimbArray::new(limbs)))
}

macro_rules! impl_array_encoding {
    ($($unit:ty),*) => {$(
        /// Two's complement over the full width of `N` limbs. Constant time,
        /// except that an input which does not fit is reported once all of
        /// it is read.
        impl<const N: usize> ArrayEncoding<$unit> for FixedBigInt<N> {
            type DecodeError = ConversionError;

            fn from_le(input: &[$unit]) -> Result<Self, ConversionError> {
                decode(input.len(), |index| input[index])
            }

            fn from_be(input: &[$unit]) -> Result<Self, ConversionError> {
                decode(input.len(), |index| input[input.len() - 1 - index])
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
                encoding::units_for_bits::<$unit>(N * Word::BITS as usize)
            }
        }
    )*};
}

impl_array_encoding!(u8, u32, u64);

#[cfg(test)]
mod tests {
    use super::FixedBigInt;
    use crate::{ArrayEncoding, Limb, Word};

    const WORD_BYTES: usize = Word::BITS as usize / 8;

    #[test]
    fn negative_inputs_are_sign_extended() {
        let value = FixedBigInt::<2>::from_le(&[0xfeu8]).unwrap();
        assert_eq!(value.as_limbs(), FixedBigInt::<2>::from(-2i8).as_limbs());
        let value = FixedBigInt::<2>::from_be(&[0xffu8, 0x7f]).unwrap();
        assert_eq!(value.as_limbs(), FixedBigInt::<2>::from(-129i16).as_limbs());
    }

    #[test]
    fn an_input_wider_than_n_limbs_must_repeat_the_sign() {
        let mut buffer = [0xffu8; 16];
        let input = &mut buffer[..WORD_BYTES + 1];
        assert_eq!(
            FixedBigInt::<1>::from_le(input).unwrap().as_limbs(),
            [Limb::new(Word::MAX)]
        );
        // a negative top byte above a positive limb is lost precision
        input.fill(0);
        input[WORD_BYTES] = 0xff;
        assert!(FixedBigInt::<1>::from_le(input).is_err());
        // a positive value whose top kept bit is set would read as negative
        input.fill(0);
        input[WORD_BYTES - 1] = 0x80;
        assert!(FixedBigInt::<1>::from_le(input).is_err());
        assert!(FixedBigInt::<2>::from_le(input).is_ok());
    }

    #[test]
    fn the_writer_sign_extends_to_the_full_width() {
        let value = FixedBigInt::<1>::from(-2i8);
        let mut output = [0u8; 8];
        let output = &mut output[..WORD_BYTES];
        assert_eq!(value.write_le(output), Ok(WORD_BYTES));
        assert_eq!(output[0], 0xfe);
        assert!(output[1..].iter().all(|&byte| byte == 0xff));
        assert_eq!(value.write_be(output), Ok(WORD_BYTES));
        assert_eq!(output[WORD_BYTES - 1], 0xfe);
        assert!(output[..WORD_BYTES - 1].iter().all(|&byte| byte == 0xff));
    }

    #[test]
    fn zero_limbs_hold_only_zero() {
        assert!(FixedBigInt::<0>::from_le(&[0u8]).is_ok());
        assert!(FixedBigInt::<0>::from_le(&[0xffu8]).is_err());
        let value = FixedBigInt::<0>::from_le(&[] as &[u8]).unwrap();
        assert_eq!(ArrayEncoding::<u8>::length(&value), 0);
    }
}
