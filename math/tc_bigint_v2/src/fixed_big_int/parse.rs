//! Parsing of [`FixedBigInt`].

use core::str::FromStr;

use num_traits::{Num, Zero};

use super::FixedBigInt;
use crate::limb::conditional_negate;
use crate::text::{accumulate, fits_signed, split};
use crate::{ParseBigIntError, Word};

/// Reads an optional `+` or `-` and then digits in `radix`, in either case,
/// as the primitive integers do; a value past the `N` limbs is an error. A
/// radix outside `2..=36` is an error, where the primitive integers panic.
/// Variable time: only for public values.
impl<const N: usize> Num for FixedBigInt<N> {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(text: &str, radix: u32) -> Result<Self, ParseBigIntError> {
        let (negative, digits) = split(text, radix)?;
        let mut value = Self::zero();
        if !accumulate(value.limbs_mut(), digits, radix) || !fits_signed(value.as_limbs(), negative)
        {
            return Err(ParseBigIntError::Overflow);
        }
        conditional_negate(value.limbs_mut(), Word::from(negative).wrapping_neg());
        Ok(value)
    }
}

/// Decimal, through [`Num::from_str_radix`]. Variable time: only for public
/// values.
impl<const N: usize> FromStr for FixedBigInt<N> {
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

    use crate::{FixedBigInt, ParseBigIntError, Word};

    /// The limbs of 128 bits, to compare against `i128`.
    const LIMBS: usize = (i128::BITS / Word::BITS) as usize;

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
                    FixedBigInt::<LIMBS>::from_str_radix(&text, radix),
                    Ok(FixedBigInt::<LIMBS>::from(a)),
                    "{text}"
                );
            }
            assert_eq!(
                format!("{}{a}", if a < 0 { "" } else { "+" }).parse::<FixedBigInt::<LIMBS>>(),
                Ok(FixedBigInt::<LIMBS>::from(a))
            );
        }
        assert_eq!(
            FixedBigInt::<LIMBS>::from_str_radix("zZ", 36),
            Ok(FixedBigInt::<LIMBS>::from(35i16 * 36 + 35))
        );
    }

    #[test]
    fn malformed_text_is_an_error() {
        for text in [
            "", "+", "-", "12a", "1_000", " 1", "1 ", "0x10", "++1", "--1",
        ] {
            let read = FixedBigInt::<LIMBS>::from_str_radix(text, 10);
            assert_eq!(read, Err(ParseBigIntError::InvalidDigit), "{text:?}");
        }
        for radix in [0, 1, 37] {
            let read = FixedBigInt::<LIMBS>::from_str_radix("1", radix);
            assert_eq!(read, Err(ParseBigIntError::InvalidRadix), "{radix}");
        }
    }

    #[test]
    fn a_value_past_the_width_is_an_error() {
        let past = format!("{}", i128::MAX as u128 + 1);
        assert_eq!(
            past.parse::<FixedBigInt::<LIMBS>>(),
            Err(ParseBigIntError::Overflow)
        );
        let min = format!("-{past}");
        assert_eq!(
            min.parse::<FixedBigInt::<LIMBS>>(),
            Ok(FixedBigInt::<LIMBS>::from(i128::MIN))
        );
        assert_eq!(
            format!("-{past}0").parse::<FixedBigInt::<LIMBS>>(),
            Err(ParseBigIntError::Overflow)
        );
    }
}
