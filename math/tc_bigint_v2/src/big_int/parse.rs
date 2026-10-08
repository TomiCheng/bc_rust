//! Parsing of [`BigInt`].

use alloc::vec;
use core::str::FromStr;

use num_traits::Num;

use super::BigInt;
use crate::limb::conditional_negate;
use crate::text::{accumulate, limbs_for_digits, split};
use crate::{Limb, ParseBigIntError, Word};

/// Reads an optional `+` or `-` and then digits in `radix`, in either case,
/// as the primitive integers do; the result is trimmed. A radix outside
/// `2..=36` is an error, where the primitive integers panic. Variable time:
/// only for public values.
impl Num for BigInt {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(text: &str, radix: u32) -> Result<Self, ParseBigIntError> {
        let (negative, digits) = split(text, radix)?;
        let mut limbs = vec![Limb::new(0); limbs_for_digits(digits.len(), radix, 1)];
        // as wide as any value of that many digits, so it always fits
        let fitted = accumulate(&mut limbs, digits, radix);
        debug_assert!(fitted);
        conditional_negate(&mut limbs, Word::from(negative).wrapping_neg());
        Ok(Self::new(limbs))
    }
}

/// Decimal, through [`Num::from_str_radix`]. Variable time: only for public
/// values.
impl FromStr for BigInt {
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

    use crate::{BigInt, ParseBigIntError};

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
                    BigInt::from_str_radix(&text, radix),
                    Ok(BigInt::from(a)),
                    "{text}"
                );
            }
            assert_eq!(
                format!("{}{a}", if a < 0 { "" } else { "+" }).parse::<BigInt>(),
                Ok(BigInt::from(a))
            );
        }
        assert_eq!(
            BigInt::from_str_radix("zZ", 36),
            Ok(BigInt::from(35i16 * 36 + 35))
        );
    }

    #[test]
    fn malformed_text_is_an_error() {
        for text in [
            "", "+", "-", "12a", "1_000", " 1", "1 ", "0x10", "++1", "--1",
        ] {
            let read = BigInt::from_str_radix(text, 10);
            assert_eq!(read, Err(ParseBigIntError::InvalidDigit), "{text:?}");
        }
        for radix in [0, 1, 37] {
            let read = BigInt::from_str_radix("1", radix);
            assert_eq!(read, Err(ParseBigIntError::InvalidRadix), "{radix}");
        }
    }

    #[test]
    fn long_text_reads_the_value_it_spells() {
        let text: String = ["-1"].into_iter().chain(["0"; 100]).collect();
        let mut expected = BigInt::from(-1i8);
        for _ in 0..100 {
            expected *= BigInt::from(10u8);
        }
        assert_eq!(text.parse::<BigInt>(), Ok(expected));
    }
}
