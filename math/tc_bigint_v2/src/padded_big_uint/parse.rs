//! Parsing of [`PaddedBigUint`].

use alloc::vec;
use core::str::FromStr;

use num_traits::Num;

use super::PaddedBigUint;
use crate::text::{accumulate, limbs_for_digits, split};
use crate::{Limb, ParseBigIntError};

/// Reads an optional `+` and then digits in `radix`, in either case, as the
/// primitive integers do; the width follows the length of the digits, each
/// counting as the bits of the largest digit, so it does not show the
/// value. A radix outside `2..=36` is an error, where the primitive
/// integers panic. Variable time: only for public values.
impl Num for PaddedBigUint {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(text: &str, radix: u32) -> Result<Self, ParseBigIntError> {
        let (negative, digits) = split(text, radix)?;
        if negative {
            return Err(ParseBigIntError::NegativeUnsigned);
        }
        let mut limbs = vec![Limb::new(0); limbs_for_digits(digits.len(), radix, 0)];
        // as wide as any value of that many digits, so it always fits
        let fitted = accumulate(&mut limbs, digits, radix);
        debug_assert!(fitted);
        Ok(Self::new(limbs.into_boxed_slice()))
    }
}

/// Decimal, through [`Num::from_str_radix`]. Variable time: only for public
/// values.
impl FromStr for PaddedBigUint {
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

    use crate::{PaddedBigUint, ParseBigIntError, Word};

    const VALUES: [u128; 7] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX,
    ];

    #[test]
    fn every_radix_reads_back_what_the_primitive_ones_print() {
        for a in VALUES {
            let texts = [
                (format!("{a:b}"), 2),
                (format!("{a:o}"), 8),
                (format!("{a}"), 10),
                (format!("{a:x}"), 16),
                (format!("{a:X}"), 16),
            ];
            for (text, radix) in texts {
                assert_eq!(
                    PaddedBigUint::from_str_radix(&text, radix),
                    Ok(PaddedBigUint::from(a)),
                    "{text}"
                );
            }
            assert_eq!(
                format!("+{a}").parse::<PaddedBigUint>(),
                Ok(PaddedBigUint::from(a))
            );
        }
        assert_eq!(
            PaddedBigUint::from_str_radix("zZ", 36),
            Ok(PaddedBigUint::from(35u16 * 36 + 35))
        );
    }

    #[test]
    fn malformed_text_is_an_error() {
        for text in [
            "", "+", "-", "12a", "1_000", " 1", "1 ", "0x10", "++1", "--1",
        ] {
            let read = PaddedBigUint::from_str_radix(text, 10);
            assert_eq!(read, Err(ParseBigIntError::InvalidDigit), "{text:?}");
        }
        for radix in [0, 1, 37] {
            let read = PaddedBigUint::from_str_radix("1", radix);
            assert_eq!(read, Err(ParseBigIntError::InvalidRadix), "{radix}");
        }
    }

    #[test]
    fn a_minus_sign_is_an_error_for_an_unsigned_type() {
        for text in ["-1", "-0"] {
            let read = PaddedBigUint::from_str_radix(text, 10);
            assert_eq!(read, Err(ParseBigIntError::NegativeUnsigned), "{text}");
        }
    }

    #[test]
    fn the_width_follows_the_length_of_the_digits() {
        let width = |bits: usize| bits.div_ceil(Word::BITS as usize);
        let short = PaddedBigUint::from_str_radix("0001", 10).unwrap();
        assert_eq!(short.as_limbs().len(), width(4 * 4));
        let long: String = ["1"].into_iter().chain(["0"; 39]).collect();
        let long = PaddedBigUint::from_str_radix(&long, 16).unwrap();
        assert_eq!(long.as_limbs().len(), width(40 * 4));
    }
}
