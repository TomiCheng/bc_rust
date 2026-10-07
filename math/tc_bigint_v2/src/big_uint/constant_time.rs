//! Constant-time comparison of [`BigUint`].

use tc_constant_time::{Choice, ConstantTimeEq};

use super::BigUint;
use crate::limb::ct_eq_extended;

/// Constant time for operands of equal length; the lengths themselves
/// follow the values, as both are trimmed.
impl ConstantTimeEq for BigUint {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        ct_eq_extended(self.as_limbs(), 0, rhs.as_limbs(), 0)
    }
}

#[cfg(test)]
mod tests {
    use tc_constant_time::ConstantTimeEq;

    use super::BigUint;

    fn equal<T: ConstantTimeEq>(a: &T, b: &T) -> bool {
        a.ct_eq(b).unwrap_u8() == 1
    }

    #[test]
    fn equal_values_compare_equal_and_others_do_not() {
        assert!(equal(&BigUint::from(300u16), &BigUint::from(300u128)));
        assert!(!equal(&BigUint::from(300u16), &BigUint::from(301u16)));
        assert!(equal(&BigUint::from(0u8), &BigUint::default()));
    }

    #[test]
    fn values_of_different_lengths_differ() {
        assert!(!equal(&BigUint::from(1u8), &BigUint::from(1u128 << 100)));
    }
}
