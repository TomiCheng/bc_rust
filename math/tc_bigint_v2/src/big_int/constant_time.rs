//! Constant-time comparison of [`BigInt`].

use core::cmp::Ordering;
use tc_constant_time::{Choice, ConstantTimeEq, ConstantTimeOrd};

use super::BigInt;
use crate::encoding::sign_fill;
use crate::limb::{ct_eq_extended, ct_lt_extended};

/// Constant time for operands of equal length; the lengths themselves
/// follow the values, as both are trimmed.
impl ConstantTimeEq for BigInt {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        let (left, right) = (self.as_limbs(), rhs.as_limbs());
        ct_eq_extended(left, sign_fill(left), right, sign_fill(right))
    }
}

/// Constant time for operands of equal length; the lengths themselves
/// follow the values, as both are trimmed.
impl ConstantTimeOrd for BigInt {
    fn ct_lt(&self, rhs: &Self) -> Choice {
        let (left, right) = (self.as_limbs(), rhs.as_limbs());
        ct_lt_extended(left, sign_fill(left), right, sign_fill(right), true)
    }
}

/// Goes through [`ConstantTimeOrd::ct_lt`] and `ct_eq`.
impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Goes through [`ConstantTimeOrd::ct_lt`] and `ct_eq`.
impl Ord for BigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        let less = self.ct_lt(other).unwrap_u8() == 1;
        let equal = self.ct_eq(other).unwrap_u8() == 1;
        match (less, equal) {
            (true, _) => Ordering::Less,
            (false, true) => Ordering::Equal,
            (false, false) => Ordering::Greater,
        }
    }
}

#[cfg(test)]
mod tests {
    use tc_constant_time::{ConstantTimeEq, ConstantTimeOrd};

    use super::BigInt;
    use crate::Word;

    const VALUES: [i128; 10] = [
        i128::MIN,
        i64::MIN as i128 - 1,
        -129,
        -1,
        0,
        1,
        255,
        u64::MAX as i128,
        u64::MAX as i128 + 1,
        i128::MAX,
    ];

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

    #[test]
    fn ordering_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                assert_eq!(BigInt::from(a).cmp(&BigInt::from(b)), a.cmp(&b), "{a} {b}");
            }
        }
    }

    #[test]
    fn the_constant_time_comparisons_agree_with_the_ordering() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigInt::from(a), BigInt::from(b));
                assert_eq!(x.ct_lt(&y).unwrap_u8() == 1, a < b, "{a} {b}");
                assert_eq!(x.ct_gt(&y).unwrap_u8() == 1, a > b, "{a} {b}");
                assert_eq!(x.ct_le(&y).unwrap_u8() == 1, a <= b, "{a} {b}");
                assert_eq!(x.ct_ge(&y).unwrap_u8() == 1, a >= b, "{a} {b}");
            }
        }
    }

    #[test]
    fn a_longer_negative_value_is_smaller() {
        assert!(BigInt::from(i128::MIN) < BigInt::from(-1i8));
        assert!(BigInt::from(-1i8) < BigInt::from(Word::MAX));
    }
}
