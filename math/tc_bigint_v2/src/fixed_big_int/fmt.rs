//! Formatting of [`FixedBigInt`].

use core::fmt;

use super::FixedBigInt;

/// Prints only the type and its width, never the value, so a secret cannot
/// reach a log or a panic message. Constant time.
impl<const N: usize> fmt::Debug for FixedBigInt<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FixedBigInt")
            .field("limbs", &self.as_limbs().len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::format;

    use super::FixedBigInt;

    #[test]
    fn debug_prints_the_type_and_the_width() {
        assert_eq!(
            format!("{:?}", FixedBigInt::<2>::from(-1i8)),
            "FixedBigInt { limbs: 2, .. }"
        );
    }

    #[test]
    fn debug_never_prints_the_value() {
        let secret = FixedBigInt::<1>::from(0x1234_5678i32);
        let printed = format!("{secret:?} {secret:#?}");
        assert!(!printed.contains("305419896"));
        assert!(!printed.contains("12345678"));
    }
}
