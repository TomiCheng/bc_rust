//! Formatting of [`PaddedBigUint`].

use core::fmt;

use super::PaddedBigUint;

/// Prints only the type and its width, never the value, so a secret cannot
/// reach a log or a panic message. Constant time.
impl fmt::Debug for PaddedBigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PaddedBigUint")
            .field("limbs", &self.as_limbs().len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::format;

    use super::PaddedBigUint;
    use crate::Word;

    #[test]
    fn debug_prints_the_type_and_the_width() {
        let limbs = u128::BITS / Word::BITS;
        assert_eq!(
            format!("{:?}", PaddedBigUint::from(1u128)),
            format!("PaddedBigUint {{ limbs: {limbs}, .. }}")
        );
        assert_eq!(
            format!("{:?}", PaddedBigUint::default()),
            "PaddedBigUint { limbs: 0, .. }"
        );
    }

    #[test]
    fn debug_never_prints_the_value() {
        let secret = PaddedBigUint::from(0x1234_5678u32);
        let printed = format!("{secret:?} {secret:#?}");
        assert!(!printed.contains("305419896"));
        assert!(!printed.contains("12345678"));
    }
}
