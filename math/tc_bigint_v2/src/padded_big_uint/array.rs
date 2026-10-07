//! Unit encoding of [`PaddedBigUint`].

use core::convert::Infallible;

use super::PaddedBigUint;
use crate::encoding;
use crate::{ArrayEncoding, ConversionError, Word};

macro_rules! impl_array_encoding {
    ($($unit:ty),*) => {$(
        /// Takes the width of the input, rounded up to whole limbs, and
        /// writes the full width. Constant time.
        impl ArrayEncoding<$unit> for PaddedBigUint {
            type DecodeError = Infallible;

            fn from_le(input: &[$unit]) -> Result<Self, Infallible> {
                let limbs = encoding::decode_vec(input.len(), |index| input[index], 0);
                Ok(Self::new(limbs.into_boxed_slice()))
            }

            fn from_be(input: &[$unit]) -> Result<Self, Infallible> {
                let limbs = encoding::decode_vec(
                    input.len(),
                    |index| input[input.len() - 1 - index],
                    0,
                );
                Ok(Self::new(limbs.into_boxed_slice()))
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
                encoding::units_for_bits::<$unit>(self.as_limbs().len() * Word::BITS as usize)
            }
        }
    )*};
}

impl_array_encoding!(u8, u32, u64);

#[cfg(test)]
mod tests {
    use super::PaddedBigUint;
    use crate::{ArrayEncoding, Word};

    const WORD_BYTES: usize = Word::BITS as usize / 8;

    #[test]
    fn the_width_comes_from_the_input_rounded_up_to_whole_limbs() {
        assert_eq!(
            PaddedBigUint::from_le(&[1u8, 2, 3])
                .unwrap()
                .as_limbs()
                .len(),
            1
        );
        assert_eq!(
            PaddedBigUint::from_be(&[1u64, 0]).unwrap().as_limbs().len(),
            128 / Word::BITS as usize
        );
        assert!(
            PaddedBigUint::from_le(&[] as &[u32])
                .unwrap()
                .as_limbs()
                .is_empty()
        );
    }

    #[test]
    fn the_writer_emits_the_full_width() {
        let value = PaddedBigUint::from_le(&[1u8, 2, 3]).unwrap();
        let bytes = ArrayEncoding::<u8>::to_le(&value);
        assert_eq!(bytes.len(), WORD_BYTES);
        assert_eq!(bytes[..4], [1, 2, 3, 0]);
        let value = PaddedBigUint::from_be(&[1u64, 0]).unwrap();
        assert_eq!(ArrayEncoding::<u64>::to_be(&value), [1, 0]);
    }
}
