//! Formatting of [`FixedBigUint`].

use core::fmt;

use super::FixedBigUint;
use crate::text::{decimal_chunks, write_decimal, write_digits};

/// Prints only the type and its width, never the value, so a secret cannot
/// reach a log or a panic message. Constant time.
impl<const N: usize> fmt::Debug for FixedBigUint<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FixedBigUint")
            .field("limbs", &self.as_limbs().len())
            .finish_non_exhaustive()
    }
}

/// Decimal, padded as the formatter asks, as for the primitive integers.
/// Variable time: only for public values, as the digits show the value.
impl<const N: usize> fmt::Display for FixedBigUint<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (negative, mut magnitude) = (false, self.clone().into_limbs());
        let mut chunks = [[0; 2]; N];
        let count = decimal_chunks(magnitude.as_mut_slice(), chunks.as_flattened_mut());
        write_decimal(f, negative, &chunks.as_flattened()[..count])
    }
}

/// Binary, with `0b` under `#`, padded as the formatter asks, as for the
/// primitive integers. Variable time: only for public values, as the digits
/// show the value.
impl<const N: usize> fmt::Binary for FixedBigUint<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_digits(f, false, self.as_limbs(), 1, "0b", false)
    }
}

/// Octal, with `0o` under `#`, padded as the formatter asks, as for the
/// primitive integers. Variable time: only for public values, as the digits
/// show the value.
impl<const N: usize> fmt::Octal for FixedBigUint<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_digits(f, false, self.as_limbs(), 3, "0o", false)
    }
}

/// Hexadecimal in small letters, with `0x` under `#`, padded as the
/// formatter asks, as for the primitive integers. Variable time: only for
/// public values, as the digits show the value.
impl<const N: usize> fmt::LowerHex for FixedBigUint<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_digits(f, false, self.as_limbs(), 4, "0x", false)
    }
}

/// Hexadecimal in capital letters, with `0x` under `#`, padded as the
/// formatter asks, as for the primitive integers. Variable time: only for
/// public values, as the digits show the value.
impl<const N: usize> fmt::UpperHex for FixedBigUint<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_digits(f, false, self.as_limbs(), 4, "0x", true)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::format;

    use crate::Word;

    use super::FixedBigUint;

    #[test]
    fn debug_prints_the_type_and_the_width() {
        assert_eq!(
            format!("{:?}", FixedBigUint::<4>::from(42u8)),
            "FixedBigUint { limbs: 4, .. }"
        );
    }

    #[test]
    fn debug_never_prints_the_value() {
        let secret = FixedBigUint::<1>::from(0x1234_5678u32);
        let printed = format!("{secret:?} {secret:#?}");
        assert!(!printed.contains("305419896"));
        assert!(!printed.contains("12345678"));
    }

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
    fn decimal_prints_as_the_primitive_ones_do() {
        for a in VALUES {
            assert_eq!(
                format!("{}", FixedBigUint::<LIMBS>::from(a)),
                format!("{a}")
            );
        }
    }

    #[test]
    fn the_other_radices_print_as_the_primitive_ones_do() {
        for a in VALUES {
            let x = FixedBigUint::<LIMBS>::from(a);
            assert_eq!(format!("{x:b}"), format!("{a:b}"));
            assert_eq!(format!("{x:o}"), format!("{a:o}"));
            assert_eq!(format!("{x:x}"), format!("{a:x}"));
            assert_eq!(format!("{x:X}"), format!("{a:X}"));
        }
    }

    #[test]
    fn the_flags_pad_as_they_do_for_the_primitive_ones() {
        let (x, a) = (FixedBigUint::<LIMBS>::from(255u8), 255u128);
        assert_eq!(format!("{x:+}"), format!("{a:+}"));
        assert_eq!(format!("{x:>8}"), format!("{a:>8}"));
        assert_eq!(format!("{x:<8}"), format!("{a:<8}"));
        assert_eq!(format!("{x:^9}"), format!("{a:^9}"));
        assert_eq!(format!("{x:*^10}"), format!("{a:*^10}"));
        assert_eq!(format!("{x:08}"), format!("{a:08}"));
        assert_eq!(format!("{x:+08}"), format!("{a:+08}"));
        assert_eq!(format!("{x:2}"), format!("{a:2}"));
        assert_eq!(format!("{x:#x}"), format!("{a:#x}"));
        assert_eq!(format!("{x:#010x}"), format!("{a:#010x}"));
        assert_eq!(format!("{x:#b}"), format!("{a:#b}"));
        assert_eq!(format!("{x:#o}"), format!("{a:#o}"));
        assert_eq!(format!("{x:+#012X}"), format!("{a:+#012X}"));
        assert_eq!(format!("{x:<#8x}"), format!("{a:<#8x}"));
    }

    #[test]
    fn no_limbs_print_as_zero() {
        let empty = FixedBigUint::<0>::default();
        assert_eq!(format!("{empty} {empty:x} {empty:#b}"), "0 0 0b0");
    }
}
