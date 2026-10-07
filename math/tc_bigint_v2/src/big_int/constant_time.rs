//! Constant-time comparison of [`BigInt`].

use tc_constant_time::{Choice, ConstantTimeEq};

use super::BigInt;
use crate::encoding::sign_fill;
use crate::limb::ct_eq_extended;

/// Constant time for operands of equal length; the lengths themselves
/// follow the values, as both are trimmed.
impl ConstantTimeEq for BigInt {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        let (left, right) = (self.as_limbs(), rhs.as_limbs());
        ct_eq_extended(left, sign_fill(left), right, sign_fill(right))
    }
}

#[cfg(test)]
mod tests {
    use tc_constant_time::ConstantTimeEq;

    use super::BigInt;
    use crate::Word;

    fn equal<T: ConstantTimeEq>(a: &T, b: &T) -> bool {
        a.ct_eq(b).unwrap_u8() == 1
    }

    #[test]
    fn equal_values_from_different_sources_compare_equal() {
        assert!(equal(&BigInt::from(-300i64), &BigInt::from(-300i16)));
        assert!(!equal(&BigInt::from(-300i64), &BigInt::from(300i16)));
        assert!(equal(&BigInt::from(0i8), &BigInt::default()));
    }

    #[test]
    fn minus_one_differs_from_the_all_ones_positive_value() {
        assert!(!equal(&BigInt::from(-1i8), &BigInt::from(Word::MAX)));
    }
}
