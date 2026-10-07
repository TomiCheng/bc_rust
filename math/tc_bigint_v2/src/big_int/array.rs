//! Unit encoding of [`BigInt`].

use core::convert::Infallible;

use super::BigInt;
use crate::encoding;
use crate::{ArrayEncoding, ConversionError, Word};

/// The bits of the shortest two's complement form, sign bit included; zero
/// has none.
fn twos_complement_bits(value: &BigInt) -> usize {
    let limbs = value.as_limbs();
    let invert = encoding::sign_fill(limbs);
    // the highest bit that differs from the sign, plus the sign bit
    for (index, limb) in limbs.iter().enumerate().rev() {
        let word = limb.to_word() ^ invert;
        if word != 0 {
            return (index + 1) * Word::BITS as usize - word.leading_zeros() as usize + 1;
        }
    }
    // no limbs is zero; limbs that only repeat the sign are -1
    match limbs.is_empty() {
        true => 0,
        false => 1,
    }
}

macro_rules! impl_array_encoding {
    ($($unit:ty),*) => {$(
        /// Two's complement in the shortest form; zero has no units.
        /// Variable time: only for public values.
        impl ArrayEncoding<$unit> for BigInt {
            type DecodeError = Infallible;

            fn from_le(input: &[$unit]) -> Result<Self, Infallible> {
                let unit = |index| input[index];
                let fill = encoding::input_fill(input.len(), unit);
                Ok(Self::new(encoding::decode_vec(input.len(), unit, fill)))
            }

            fn from_be(input: &[$unit]) -> Result<Self, Infallible> {
                let unit = |index| input[input.len() - 1 - index];
                let fill = encoding::input_fill(input.len(), unit);
                Ok(Self::new(encoding::decode_vec(input.len(), unit, fill)))
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
                encoding::units_for_bits::<$unit>(twos_complement_bits(self))
            }
        }
    )*};
}

impl_array_encoding!(u8, u32, u64);

#[cfg(test)]
mod tests {
    use super::BigInt;
    use crate::{ArrayEncoding, Word};

    #[test]
    fn values_are_written_in_the_shortest_twos_complement() {
        let bytes = |value: i128| ArrayEncoding::<u8>::to_le(&BigInt::from(value));
        assert_eq!(bytes(-2), [0xfe]);
        assert_eq!(bytes(127), [0x7f]);
        assert_eq!(bytes(128), [0x80, 0x00]);
        assert_eq!(bytes(-128), [0x80]);
        assert_eq!(bytes(-1), [0xff]);
        assert_eq!(
            ArrayEncoding::<u8>::to_be(&BigInt::from(-129i16)),
            [0xff, 0x7f]
        );
        assert_eq!(ArrayEncoding::<u32>::to_le(&BigInt::from(-1i8)), [u32::MAX]);
    }

    #[test]
    fn zero_encodes_to_no_units() {
        assert!(ArrayEncoding::<u8>::to_le(&BigInt::from(0i8)).is_empty());
    }

    #[test]
    fn a_positive_value_with_its_top_bit_set_gets_a_zero_unit_above_it() {
        let words = ArrayEncoding::<u64>::to_le(&BigInt::from(Word::MAX));
        match Word::BITS {
            64 => assert_eq!(words, [u64::MAX, 0]),
            _ => assert_eq!(words, [u32::MAX as u64]),
        }
    }

    #[test]
    fn decoding_reads_the_top_bit_as_the_sign() {
        assert_eq!(
            BigInt::from_le(&[0x80u8]).unwrap().as_limbs(),
            BigInt::from(-128i8).as_limbs()
        );
        assert_eq!(
            BigInt::from_be(&[0u8, 0x80]).unwrap().as_limbs(),
            BigInt::from(128u8).as_limbs()
        );
    }

    #[test]
    fn values_round_trip_and_match_the_primitive_encoding() {
        for value in [
            i128::MIN,
            -0x1234_5678_9abc,
            -0x81,
            -1,
            0,
            1,
            0x80,
            i128::MAX,
        ] {
            let integer = BigInt::from(value);
            let bytes = ArrayEncoding::<u8>::to_le(&integer);
            assert_eq!(
                BigInt::from_le(&bytes).unwrap().as_limbs(),
                integer.as_limbs()
            );
            let full = value.to_le_bytes();
            assert_eq!(
                BigInt::from_le(&full).unwrap().as_limbs(),
                integer.as_limbs()
            );
            let words = ArrayEncoding::<u32>::to_be(&integer);
            assert_eq!(
                BigInt::from_be(&words).unwrap().as_limbs(),
                integer.as_limbs()
            );
        }
        let bytes = ArrayEncoding::<u8>::to_le(&BigInt::from(i128::MIN));
        assert_eq!(bytes, i128::MIN.to_le_bytes());
    }
}
