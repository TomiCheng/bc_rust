//! Unit encoding of [`BigUint`].

use core::convert::Infallible;

use super::BigUint;
use crate::encoding;
use crate::{ArrayEncoding, ConversionError, Word};

/// The bits up to the highest set one; zero has none.
fn significant_bits(value: &BigUint) -> usize {
    let limbs = value.as_limbs();
    limbs.last().map_or(0, |top| {
        limbs.len() * Word::BITS as usize - top.to_word().leading_zeros() as usize
    })
}

macro_rules! impl_array_encoding {
    ($($unit:ty),*) => {$(
        /// Writes the shortest form; zero has no units. Variable time: only
        /// for public values.
        impl ArrayEncoding<$unit> for BigUint {
            type DecodeError = Infallible;

            fn from_le(input: &[$unit]) -> Result<Self, Infallible> {
                Ok(Self::new(encoding::decode_vec(input.len(), |index| input[index], 0)))
            }

            fn from_be(input: &[$unit]) -> Result<Self, Infallible> {
                let unit = |index| input[input.len() - 1 - index];
                Ok(Self::new(encoding::decode_vec(input.len(), unit, 0)))
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
                encoding::units_for_bits::<$unit>(significant_bits(self))
            }
        }
    )*};
}

impl_array_encoding!(u8, u32, u64);

#[cfg(test)]
mod tests {
    use super::BigUint;
    use crate::ArrayEncoding;

    #[test]
    fn zero_encodes_to_no_units() {
        assert!(ArrayEncoding::<u8>::to_le(&BigUint::from(0u8)).is_empty());
        assert!(ArrayEncoding::<u64>::to_be(&BigUint::from(0u8)).is_empty());
    }

    #[test]
    fn the_writer_emits_the_shortest_form() {
        assert_eq!(ArrayEncoding::<u8>::to_be(&BigUint::from(256u16)), [1, 0]);
        assert_eq!(ArrayEncoding::<u8>::to_le(&BigUint::from(0x80u8)), [0x80]);
        assert_eq!(
            ArrayEncoding::<u64>::to_le(&BigUint::from(u128::MAX)),
            [u64::MAX, u64::MAX]
        );
    }

    #[test]
    fn decoding_trims_leading_zero_units() {
        assert_eq!(
            BigUint::from_le(&[42u32, 0, 0]).unwrap().as_limbs(),
            BigUint::from(42u8).as_limbs()
        );
    }

    #[test]
    fn values_round_trip_through_every_unit() {
        for value in [0u128, 1, 0xff, 0x100, u64::MAX as u128, u128::MAX] {
            let value = BigUint::from(value);
            let bytes = ArrayEncoding::<u8>::to_be(&value);
            assert_eq!(
                BigUint::from_be(&bytes).unwrap().as_limbs(),
                value.as_limbs()
            );
            let words = ArrayEncoding::<u32>::to_le(&value);
            assert_eq!(
                BigUint::from_le(&words).unwrap().as_limbs(),
                value.as_limbs()
            );
            let words = ArrayEncoding::<u64>::to_be(&value);
            assert_eq!(
                BigUint::from_be(&words).unwrap().as_limbs(),
                value.as_limbs()
            );
        }
    }
}
