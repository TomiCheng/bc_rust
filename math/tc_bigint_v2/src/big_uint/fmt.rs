//! Formatting of [`BigUint`].

use core::fmt;

use super::BigUint;

/// Prints only the type and its width, never the value, so a secret cannot
/// reach a log or a panic message. Constant time.
impl fmt::Debug for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BigUint")
            .field("limbs", &self.as_limbs().len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::format;

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
}
