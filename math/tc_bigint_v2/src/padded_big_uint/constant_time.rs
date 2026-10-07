//! Constant-time comparison of [`PaddedBigUint`].

use core::cmp::Ordering;
use tc_constant_time::{Choice, ConstantTimeEq, ConstantTimeOrd};

use super::PaddedBigUint;
use crate::limb::{ct_eq_extended, ct_lt_extended};

/// Compares values, so a narrower operand counts as zero-extended.
/// Constant time: the widths only decide how far the comparison runs.
impl ConstantTimeEq for PaddedBigUint {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        ct_eq_extended(self.as_limbs(), 0, rhs.as_limbs(), 0)
    }
}

/// Compares values at any width through [`ConstantTimeEq::ct_eq`], so `==`
/// is constant time.
impl PartialEq for PaddedBigUint {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl Eq for PaddedBigUint {}

/// Compares values, so a narrower operand counts as zero-extended.
/// Constant time: the widths only decide how far the comparison runs.
impl ConstantTimeOrd for PaddedBigUint {
    fn ct_lt(&self, rhs: &Self) -> Choice {
        ct_lt_extended(self.as_limbs(), 0, rhs.as_limbs(), 0, false)
    }
}

/// Orders values at any width through [`ConstantTimeOrd::ct_lt`] and
/// `ct_eq`, so it is constant time.
impl PartialOrd for PaddedBigUint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Orders values at any width through [`ConstantTimeOrd::ct_lt`] and
/// `ct_eq`, so it is constant time.
impl Ord for PaddedBigUint {
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

    use super::PaddedBigUint;

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

    #[test]
    fn ordering_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                assert_eq!(
                    PaddedBigUint::from(a).cmp(&PaddedBigUint::from(b)),
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
                let (x, y) = (PaddedBigUint::from(a), PaddedBigUint::from(b));
                assert_eq!(x.ct_lt(&y).unwrap_u8() == 1, a < b, "{a} {b}");
                assert_eq!(x.ct_gt(&y).unwrap_u8() == 1, a > b, "{a} {b}");
                assert_eq!(x.ct_le(&y).unwrap_u8() == 1, a <= b, "{a} {b}");
                assert_eq!(x.ct_ge(&y).unwrap_u8() == 1, a >= b, "{a} {b}");
            }
        }
    }

    #[test]
    fn values_at_different_widths_are_ordered_by_value() {
        assert!(PaddedBigUint::from(2u8) > PaddedBigUint::from(1u128));
        assert!(PaddedBigUint::from(1u128) < PaddedBigUint::from(2u8));
        assert!(PaddedBigUint::from(255u8) < PaddedBigUint::from(u128::MAX));
    }
}
