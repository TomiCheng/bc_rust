//! Formatting of [`BigUint`].

use alloc::vec;
use core::fmt;

use super::BigUint;
use crate::text::{decimal_chunks, write_decimal, write_digits};

/// Prints only the type and its width, never the value, so a secret cannot
/// reach a log or a panic message. Constant time.
impl fmt::Debug for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BigUint")
            .field("limbs", &self.as_limbs().len())
            .finish_non_exhaustive()
    }
}

/// Decimal, padded as the formatter asks, as for the primitive integers.
/// Variable time: only for public values, as the digits show the value.
impl fmt::Display for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (negative, mut magnitude) = (false, self.as_limbs().to_vec());
        let mut chunks = vec![0; 2 * magnitude.len()];
        let count = decimal_chunks(&mut magnitude, &mut chunks);
        write_decimal(f, negative, &chunks[..count])
    }
}

/// Binary, with `0b` under `#`, padded as the formatter asks, as for the
/// primitive integers. Variable time: only for public values, as the digits
/// show the value.
impl fmt::Binary for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_digits(f, false, self.as_limbs(), 1, "0b", false)
    }
}

/// Octal, with `0o` under `#`, padded as the formatter asks, as for the
/// primitive integers. Variable time: only for public values, as the digits
/// show the value.
impl fmt::Octal for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_digits(f, false, self.as_limbs(), 3, "0o", false)
    }
}

/// Hexadecimal in small letters, with `0x` under `#`, padded as the
/// formatter asks, as for the primitive integers. Variable time: only for
/// public values, as the digits show the value.
impl fmt::LowerHex for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_digits(f, false, self.as_limbs(), 4, "0x", false)
    }
}

/// Hexadecimal in capital letters, with `0x` under `#`, padded as the
/// formatter asks, as for the primitive integers. Variable time: only for
/// public values, as the digits show the value.
impl fmt::UpperHex for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_digits(f, false, self.as_limbs(), 4, "0x", true)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::format;
    use std::string::String;

    use super::BigUint;

    #[test]
    fn debug_prints_the_type_and_the_width() {
        assert_eq!(
            format!("{:?}", BigUint::from(42u8)),
            "BigUint { limbs: 1, .. }"
        );
        assert_eq!(
            format!("{:?}", BigUint::from(0u8)),
            "BigUint { limbs: 0, .. }"
        );
    }

    #[test]
    fn debug_never_prints_the_value() {
        let secret = BigUint::from(0x1234_5678u32);
        let printed = format!("{secret:?} {secret:#?}");
        assert!(!printed.contains("305419896"));
        assert!(!printed.contains("12345678"));
    }

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
            assert_eq!(format!("{}", BigUint::from(a)), format!("{a}"));
        }
    }

    #[test]
    fn the_other_radices_print_as_the_primitive_ones_do() {
        for a in VALUES {
            let x = BigUint::from(a);
            assert_eq!(format!("{x:b}"), format!("{a:b}"));
            assert_eq!(format!("{x:o}"), format!("{a:o}"));
            assert_eq!(format!("{x:x}"), format!("{a:x}"));
            assert_eq!(format!("{x:X}"), format!("{a:X}"));
        }
    }

    #[test]
    fn the_flags_pad_as_they_do_for_the_primitive_ones() {
        let (x, a) = (BigUint::from(255u8), 255u128);
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
    fn long_values_print_and_read_back() {
        let x = BigUint::from(1u8) << 4000;
        let hex: String = ["1"].into_iter().chain(["0"; 1000]).collect();
        assert_eq!(format!("{x:x}"), hex);
        assert_eq!(format!("{x}").parse::<BigUint>(), Ok(x));
    }
}
