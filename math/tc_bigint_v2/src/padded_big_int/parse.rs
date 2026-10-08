//! Parsing of [`PaddedBigInt`].

use alloc::vec;
use core::str::FromStr;

use num_traits::Num;

use super::PaddedBigInt;
use crate::limb::conditional_negate;
use crate::text::{accumulate, limbs_for_digits, split};
use crate::{Limb, ParseBigIntError, Word};

/// Reads an optional `+` or `-` and then digits in `radix`, in either case,
/// as the primitive integers do; the width follows the length of the
/// digits, each counting as the bits of the largest digit, with one bit
/// more for the sign, so it does not show the value. A radix outside
/// `2..=36` is an error, where the primitive integers panic. Variable time:
/// only for public values.
impl Num for PaddedBigInt {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(text: &str, radix: u32) -> Result<Self, ParseBigIntError> {
        let (negative, digits) = split(text, radix)?;
        let mut limbs = vec![Limb::new(0); limbs_for_digits(digits.len(), radix, 1)];
        // as wide as any value of that many digits, so it always fits
        let fitted = accumulate(&mut limbs, digits, radix);
        debug_assert!(fitted);
        conditional_negate(&mut limbs, Word::from(negative).wrapping_neg());
        Ok(Self::new(limbs.into_boxed_slice()))
    }
}

/// Decimal, through [`Num::from_str_radix`]. Variable time: only for public
/// values.
impl FromStr for PaddedBigInt {
    type Err = ParseBigIntError;

    fn from_str(text: &str) -> Result<Self, ParseBigIntError> {
        Self::from_str_radix(text, 10)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::format;
    use std::string::String;

    use num_traits::Num;

    use crate::{PaddedBigInt, ParseBigIntError, Word};

    const VALUES: [i128; 9] = [
        i128::MIN,
        i64::MIN as i128 - 1,
        -129,
        -1,
        0,
        1,
        255,
        u64::MAX as i128,
        i128::MAX,
    ];

    #[test]
    fn every_radix_reads_back_what_the_primitive_ones_print() {
        for a in VALUES {
            let (sign, magnitude) = match a < 0 {
                true => ("-", a.unsigned_abs()),
                false => ("", a as u128),
            };
            let texts = [
                (format!("{sign}{magnitude:b}"), 2),
                (format!("{sign}{magnitude:o}"), 8),
                (format!("{a}"), 10),
                (format!("{sign}{magnitude:x}"), 16),
                (format!("{sign}{magnitude:X}"), 16),
            ];
            for (text, radix) in texts {
                assert_eq!(
                    PaddedBigInt::from_str_radix(&text, radix),
                    Ok(PaddedBigInt::from(a)),
                    "{text}"
                );
            }
            assert_eq!(
                format!("{}{a}", if a < 0 { "" } else { "+" }).parse::<PaddedBigInt>(),
                Ok(PaddedBigInt::from(a))
            );
        }
        assert_eq!(
            PaddedBigInt::from_str_radix("zZ", 36),
            Ok(PaddedBigInt::from(35i16 * 36 + 35))
        );
    }

    #[test]
    fn malformed_text_is_an_error() {
        for text in [
            "", "+", "-", "12a", "1_000", " 1", "1 ", "0x10", "++1", "--1",
        ] {
            let read = PaddedBigInt::from_str_radix(text, 10);
            assert_eq!(read, Err(ParseBigIntError::InvalidDigit), "{text:?}");
        }
        for radix in [0, 1, 37] {
            let read = PaddedBigInt::from_str_radix("1", radix);
            assert_eq!(read, Err(ParseBigIntError::InvalidRadix), "{radix}");
        }
    }

    #[test]
    fn the_width_follows_the_length_of_the_digits() {
        let width = |bits: usize| bits.div_ceil(Word::BITS as usize);
        let short = PaddedBigInt::from_str_radix("0001", 10).unwrap();
        assert_eq!(short.as_limbs().len(), width(4 * 4 + 1));
        let long: String = ["1"].into_iter().chain(["0"; 39]).collect();
        let long = PaddedBigInt::from_str_radix(&long, 16).unwrap();
        assert_eq!(long.as_limbs().len(), width(40 * 4 + 1));
    }
}
