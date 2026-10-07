//! Constant-time comparison of [`PaddedBigUint`].

use tc_constant_time::{Choice, ConstantTimeEq};

use super::PaddedBigUint;
use crate::limb::ct_eq_extended;

/// Compares values, so a narrower operand counts as zero-extended.
/// Constant time: the widths only decide how far the comparison runs.
impl ConstantTimeEq for PaddedBigUint {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        ct_eq_extended(self.as_limbs(), 0, rhs.as_limbs(), 0)
    }
}

#[cfg(test)]
mod tests {
    use tc_constant_time::ConstantTimeEq;

    use super::PaddedBigUint;

    fn equal<T: ConstantTimeEq>(a: &T, b: &T) -> bool {
        a.ct_eq(b).unwrap_u8() == 1
    }

    #[test]
    fn values_of_different_widths_compare_by_value() {
        assert!(equal(
            &PaddedBigUint::from(1u8),
            &PaddedBigUint::from(1u128)
        ));
        assert!(equal(
            &PaddedBigUint::from(1u128),
            &PaddedBigUint::from(1u8)
        ));
        assert!(!equal(
            &PaddedBigUint::from(1u8),
            &PaddedBigUint::from(2u128)
        ));
    }

    #[test]
    fn a_set_bit_beyond_the_narrower_width_is_a_difference() {
        assert!(!equal(
            &PaddedBigUint::from(u64::MAX),
            &PaddedBigUint::from(u128::MAX)
        ));
    }

    #[test]
    fn zero_width_equals_zero() {
        assert!(equal(&PaddedBigUint::default(), &PaddedBigUint::from(0u64)));
    }
}
