//! Constant-time comparison of [`BigUint`].

use core::cmp::Ordering;
use tc_constant_time::{Choice, ConstantTimeEq, ConstantTimeOrd};

use super::BigUint;
use crate::limb::{ct_eq_extended, ct_lt_extended};

/// Constant time for operands of equal length; the lengths themselves
/// follow the values, as both are trimmed.
impl ConstantTimeEq for BigUint {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        ct_eq_extended(self.as_limbs(), 0, rhs.as_limbs(), 0)
    }
}

/// Constant time for operands of equal length; the lengths themselves
/// follow the values, as both are trimmed.
impl ConstantTimeOrd for BigUint {
    fn ct_lt(&self, rhs: &Self) -> Choice {
        ct_lt_extended(self.as_limbs(), 0, rhs.as_limbs(), 0, false)
    }
}

/// Goes through [`ConstantTimeOrd::ct_lt`] and `ct_eq`. Variable time, as the
/// trimmed lengths follow the values: only for public values.
impl PartialOrd for BigUint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Goes through [`ConstantTimeOrd::ct_lt`] and `ct_eq`. Variable time, as the
/// trimmed lengths follow the values: only for public values.
impl Ord for BigUint {
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

    use super::BigUint;

    const VALUES: [u128; 8] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX - 1,
        u128::MAX,
    ];

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

    #[test]
    fn ordering_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                assert_eq!(
                    BigUint::from(a).cmp(&BigUint::from(b)),
                    a.cmp(&b),
                    "{a} {b}"
                );
            }
        }
    }

    #[test]
    fn the_constant_time_comparisons_agree_with_the_ordering() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigUint::from(a), BigUint::from(b));
                assert_eq!(x.ct_lt(&y).unwrap_u8() == 1, a < b, "{a} {b}");
                assert_eq!(x.ct_gt(&y).unwrap_u8() == 1, a > b, "{a} {b}");
                assert_eq!(x.ct_le(&y).unwrap_u8() == 1, a <= b, "{a} {b}");
                assert_eq!(x.ct_ge(&y).unwrap_u8() == 1, a >= b, "{a} {b}");
            }
        }
    }

    #[test]
    fn a_longer_value_is_larger() {
        assert!(BigUint::from(1u128 << 100) > BigUint::from(u64::MAX));
    }
}
