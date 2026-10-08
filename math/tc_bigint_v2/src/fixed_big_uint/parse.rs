//! Parsing of [`FixedBigUint`].

use core::str::FromStr;

use num_traits::{Num, Zero};

use super::FixedBigUint;
use crate::ParseBigIntError;
use crate::text::{accumulate, split};

/// Reads an optional `+` and then digits in `radix`, in either case, as the
/// primitive integers do; a value past the `N` limbs is an error. A radix
/// outside `2..=36` is an error, where the primitive integers panic.
/// Variable time: only for public values.
impl<const N: usize> Num for FixedBigUint<N> {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(text: &str, radix: u32) -> Result<Self, ParseBigIntError> {
        let (negative, digits) = split(text, radix)?;
        if negative {
            return Err(ParseBigIntError::NegativeUnsigned);
        }
        let mut value = Self::zero();
        if !accumulate(value.limbs_mut(), digits, radix) {
            return Err(ParseBigIntError::Overflow);
        }
        Ok(value)
    }
}

/// Decimal, through [`Num::from_str_radix`]. Variable time: only for public
/// values.
impl<const N: usize> FromStr for FixedBigUint<N> {
    type Err = ParseBigIntError;

    fn from_str(text: &str) -> Result<Self, ParseBigIntError> {
        Self::from_str_radix(text, 10)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::format;

    use num_traits::Num;

    use crate::{FixedBigUint, ParseBigIntError, Word};

    /// The limbs of 128 bits, to compare against `u128`.
    const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

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
                    FixedBigUint::<LIMBS>::from_str_radix(&text, radix),
                    Ok(FixedBigUint::<LIMBS>::from(a)),
                    "{text}"
                );
            }
            assert_eq!(
                format!("+{a}").parse::<FixedBigUint::<LIMBS>>(),
                Ok(FixedBigUint::<LIMBS>::from(a))
            );
        }
        assert_eq!(
            FixedBigUint::<LIMBS>::from_str_radix("zZ", 36),
            Ok(FixedBigUint::<LIMBS>::from(35u16 * 36 + 35))
        );
    }

    #[test]
    fn malformed_text_is_an_error() {
        for text in [
            "", "+", "-", "12a", "1_000", " 1", "1 ", "0x10", "++1", "--1",
        ] {
            let read = FixedBigUint::<LIMBS>::from_str_radix(text, 10);
            assert_eq!(read, Err(ParseBigIntError::InvalidDigit), "{text:?}");
        }
        for radix in [0, 1, 37] {
            let read = FixedBigUint::<LIMBS>::from_str_radix("1", radix);
            assert_eq!(read, Err(ParseBigIntError::InvalidRadix), "{radix}");
        }
    }

    #[test]
    fn a_minus_sign_is_an_error_for_an_unsigned_type() {
        for text in ["-1", "-0"] {
            let read = FixedBigUint::<LIMBS>::from_str_radix(text, 10);
            assert_eq!(read, Err(ParseBigIntError::NegativeUnsigned), "{text}");
        }
    }

    #[test]
    fn a_value_past_the_width_is_an_error() {
        let max = format!("{}", u128::MAX);
        assert_eq!(
            max.parse::<FixedBigUint::<LIMBS>>(),
            Ok(FixedBigUint::<LIMBS>::from(u128::MAX))
        );
        let past = format!("{max}0");
        assert_eq!(
            past.parse::<FixedBigUint::<LIMBS>>(),
            Err(ParseBigIntError::Overflow)
        );
    }
}
