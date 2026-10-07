//! Formatting of [`FixedBigUint`].

use core::fmt;

use super::FixedBigUint;

/// Prints only the type and its width, never the value, so a secret cannot
/// reach a log or a panic message. Constant time.
impl<const N: usize> fmt::Debug for FixedBigUint<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FixedBigUint")
            .field("limbs", &self.as_limbs().len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::format;

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
}
