//! Parsing of [`BigUint`].

use alloc::vec;
use core::str::FromStr;

use num_traits::Num;

use super::BigUint;
use crate::text::{accumulate, limbs_for_digits, split};
use crate::{Limb, ParseBigIntError};

/// Reads an optional `+` and then digits in `radix`, in either case, as the
/// primitive integers do; the result is trimmed. A radix outside `2..=36`
/// is an error, where the primitive integers panic. Variable time: only for
/// public values.
impl Num for BigUint {
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
        Ok(Self::new(limbs))
    }
}

/// Decimal, through [`Num::from_str_radix`]. Variable time: only for public
/// values.
impl FromStr for BigUint {
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

    use crate::{BigUint, ParseBigIntError};

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
                    BigUint::from_str_radix(&text, radix),
                    Ok(BigUint::from(a)),
                    "{text}"
                );
            }
            assert_eq!(format!("+{a}").parse::<BigUint>(), Ok(BigUint::from(a)));
        }
        assert_eq!(
            BigUint::from_str_radix("zZ", 36),
            Ok(BigUint::from(35u16 * 36 + 35))
        );
    }

    #[test]
    fn malformed_text_is_an_error() {
        for text in [
            "", "+", "-", "12a", "1_000", " 1", "1 ", "0x10", "++1", "--1",
        ] {
            let read = BigUint::from_str_radix(text, 10);
            assert_eq!(read, Err(ParseBigIntError::InvalidDigit), "{text:?}");
        }
        for radix in [0, 1, 37] {
            let read = BigUint::from_str_radix("1", radix);
            assert_eq!(read, Err(ParseBigIntError::InvalidRadix), "{radix}");
        }
    }

    #[test]
    fn a_minus_sign_is_an_error_for_an_unsigned_type() {
        for text in ["-1", "-0"] {
            let read = BigUint::from_str_radix(text, 10);
            assert_eq!(read, Err(ParseBigIntError::NegativeUnsigned), "{text}");
        }
    }

    #[test]
    fn long_text_reads_the_value_it_spells() {
        let text: String = ["1"].into_iter().chain(["0"; 100]).collect();
        let mut expected = BigUint::from(1u8);
        for _ in 0..100 {
            expected *= BigUint::from(10u8);
        }
        assert_eq!(text.parse::<BigUint>(), Ok(expected));
    }
}
