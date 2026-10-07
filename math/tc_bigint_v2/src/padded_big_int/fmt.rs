//! Formatting of [`PaddedBigInt`].

use core::fmt;

use super::PaddedBigInt;

/// Prints only the type and its width, never the value, so a secret cannot
/// reach a log or a panic message. Constant time.
impl fmt::Debug for PaddedBigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PaddedBigInt")
            .field("limbs", &self.as_limbs().len())
            .finish_non_exhaustive()
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
}
