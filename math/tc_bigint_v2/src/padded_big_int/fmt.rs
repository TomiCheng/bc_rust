//! Formatting of [`PaddedBigInt`].

use alloc::{vec, vec::Vec};
use core::fmt;

use super::PaddedBigInt;
use crate::Limb;
use crate::encoding::sign_fill;
use crate::limb::conditional_negate;
use crate::text::{decimal_chunks, write_decimal, write_digits};

/// Prints only the type and its width, never the value, so a secret cannot
/// reach a log or a panic message. Constant time.
impl fmt::Debug for PaddedBigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PaddedBigInt")
            .field("limbs", &self.as_limbs().len())
            .finish_non_exhaustive()
    }
}

impl PaddedBigInt {
    /// Whether the value is negative, and its magnitude in as many limbs, which
    /// hold even that of the most negative value as unsigned. Variable time.
    fn sign_and_magnitude(&self) -> (bool, Vec<Limb>) {
        let sign = sign_fill(self.as_limbs());
        let mut magnitude = self.as_limbs().to_vec();
        conditional_negate(&mut magnitude, sign);
        (sign != 0, magnitude)
    }
}

/// Decimal, padded as the formatter asks, as for the primitive integers.
/// Variable time: only for public values, as the digits show the value.
impl fmt::Display for PaddedBigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (negative, mut magnitude) = self.sign_and_magnitude();
        let mut chunks = vec![0; 2 * magnitude.len()];
        let count = decimal_chunks(&mut magnitude, &mut chunks);
        write_decimal(f, negative, &chunks[..count])
    }
}

/// Binary of the magnitude after a minus sign for a negative value, unlike
/// the primitive integers, which print the bits of the two's complement;
/// with `0b` under `#`, padded as the formatter asks. Variable time: only
/// for public values, as the digits show the value.
impl fmt::Binary for PaddedBigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (negative, magnitude) = self.sign_and_magnitude();
        write_digits(f, negative, &magnitude, 1, "0b", false)
    }
}

/// Octal of the magnitude after a minus sign for a negative value, unlike
/// the primitive integers, which print the bits of the two's complement;
/// with `0o` under `#`, padded as the formatter asks. Variable time: only
/// for public values, as the digits show the value.
impl fmt::Octal for PaddedBigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (negative, magnitude) = self.sign_and_magnitude();
        write_digits(f, negative, &magnitude, 3, "0o", false)
    }
}

/// Hexadecimal in small letters of the magnitude after a minus sign for a
/// negative value, unlike the primitive integers, which print the bits of
/// the two's complement; with `0x` under `#`, padded as the formatter asks.
/// Variable time: only for public values, as the digits show the value.
impl fmt::LowerHex for PaddedBigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (negative, magnitude) = self.sign_and_magnitude();
        write_digits(f, negative, &magnitude, 4, "0x", false)
    }
}

/// Hexadecimal in capital letters of the magnitude after a minus sign for a
/// negative value, unlike the primitive integers, which print the bits of
/// the two's complement; with `0x` under `#`, padded as the formatter asks.
/// Variable time: only for public values, as the digits show the value.
impl fmt::UpperHex for PaddedBigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (negative, magnitude) = self.sign_and_magnitude();
        write_digits(f, negative, &magnitude, 4, "0x", true)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::format;

    use super::PaddedBigInt;

    #[test]
    fn debug_prints_the_type_and_the_width() {
        assert_eq!(
            format!("{:?}", PaddedBigInt::from(-1i8)),
            "PaddedBigInt { limbs: 1, .. }"
        );
        assert_eq!(
            format!("{:?}", PaddedBigInt::default()),
            "PaddedBigInt { limbs: 0, .. }"
        );
    }

    #[test]
    fn debug_never_prints_the_value() {
        let secret = PaddedBigInt::from(0x1234_5678i32);
        let printed = format!("{secret:?} {secret:#?}");
        assert!(!printed.contains("305419896"));
        assert!(!printed.contains("12345678"));
    }

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
    fn decimal_prints_as_the_primitive_ones_do() {
        for a in VALUES {
            assert_eq!(format!("{}", PaddedBigInt::from(a)), format!("{a}"));
        }
    }

    #[test]
    fn the_other_radices_print_the_sign_and_the_magnitude() {
        for a in VALUES {
            let x = PaddedBigInt::from(a);
            let (sign, magnitude) = match a < 0 {
                true => ("-", a.unsigned_abs()),
                false => ("", a as u128),
            };
            assert_eq!(format!("{x:b}"), format!("{sign}{magnitude:b}"));
            assert_eq!(format!("{x:o}"), format!("{sign}{magnitude:o}"));
            assert_eq!(format!("{x:x}"), format!("{sign}{magnitude:x}"));
            assert_eq!(format!("{x:X}"), format!("{sign}{magnitude:X}"));
        }
        let negative = PaddedBigInt::from(-255i16);
        assert_eq!(format!("{negative:#x}"), "-0xff");
        assert_eq!(format!("{negative:#010x}"), "-0x00000ff");
        assert_eq!(format!("{negative:>8x}"), "     -ff");
    }

    #[test]
    fn the_flags_pad_as_they_do_for_the_primitive_ones() {
        let (x, a) = (PaddedBigInt::from(-255i16), -255i128);
        assert_eq!(format!("{x:+}"), format!("{a:+}"));
        assert_eq!(format!("{x:>8}"), format!("{a:>8}"));
        assert_eq!(format!("{x:<8}"), format!("{a:<8}"));
        assert_eq!(format!("{x:^9}"), format!("{a:^9}"));
        assert_eq!(format!("{x:*^10}"), format!("{a:*^10}"));
        assert_eq!(format!("{x:08}"), format!("{a:08}"));
        assert_eq!(format!("{x:+08}"), format!("{a:+08}"));
        assert_eq!(format!("{x:2}"), format!("{a:2}"));
    }

    #[test]
    fn width_zero_prints_as_zero() {
        let empty = PaddedBigInt::default();
        assert_eq!(format!("{empty} {empty:x} {empty:#b}"), "0 0 0b0");
    }
}
