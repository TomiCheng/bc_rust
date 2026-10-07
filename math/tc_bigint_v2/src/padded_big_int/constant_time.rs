//! Constant-time comparison of [`PaddedBigInt`].

use core::cmp::Ordering;
use tc_constant_time::{Choice, ConstantTimeEq, ConstantTimeOrd};

use super::PaddedBigInt;
use crate::encoding::sign_fill;
use crate::limb::{ct_eq_extended, ct_lt_extended};

/// Compares values, so a narrower operand counts as sign-extended.
/// Constant time: the widths only decide how far the comparison runs.
impl ConstantTimeEq for PaddedBigInt {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        let (left, right) = (self.as_limbs(), rhs.as_limbs());
        ct_eq_extended(left, sign_fill(left), right, sign_fill(right))
    }
}

/// Compares values at any width through [`ConstantTimeEq::ct_eq`], so `==`
/// is constant time.
impl PartialEq for PaddedBigInt {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl Eq for PaddedBigInt {}

/// Compares values, so a narrower operand counts as sign-extended.
/// Constant time: the widths only decide how far the comparison runs.
impl ConstantTimeOrd for PaddedBigInt {
    fn ct_lt(&self, rhs: &Self) -> Choice {
        let (left, right) = (self.as_limbs(), rhs.as_limbs());
        ct_lt_extended(left, sign_fill(left), right, sign_fill(right), true)
    }
}

/// Orders values at any width through [`ConstantTimeOrd::ct_lt`] and
/// `ct_eq`, so it is constant time.
impl PartialOrd for PaddedBigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Orders values at any width through [`ConstantTimeOrd::ct_lt`] and
/// `ct_eq`, so it is constant time.
impl Ord for PaddedBigInt {
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

    use super::PaddedBigInt;
    use crate::ArrayEncoding;

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
    fn a_narrower_operand_is_sign_extended() {
        assert!(equal(
            &PaddedBigInt::from(-1i8),
            &PaddedBigInt::from(-1i128)
        ));
        assert!(equal(&PaddedBigInt::from(5i128), &PaddedBigInt::from(5i8)));
    }

    #[test]
    fn sign_extension_is_not_mistaken_for_zeros() {
        let two_fifty_five = PaddedBigInt::from_le(&[0xffu8, 0x00]).unwrap();
        assert!(!equal(&PaddedBigInt::from(-1i8), &two_fifty_five));
        assert!(!equal(
            &PaddedBigInt::from(-1i8),
            &PaddedBigInt::from(1i128)
        ));
    }

    #[test]
    fn zero_width_equals_zero() {
        assert!(equal(&PaddedBigInt::default(), &PaddedBigInt::from(0i32)));
    }

    #[test]
    fn ordering_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                assert_eq!(
                    PaddedBigInt::from(a).cmp(&PaddedBigInt::from(b)),
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
                let (x, y) = (PaddedBigInt::from(a), PaddedBigInt::from(b));
                assert_eq!(x.ct_lt(&y).unwrap_u8() == 1, a < b, "{a} {b}");
                assert_eq!(x.ct_gt(&y).unwrap_u8() == 1, a > b, "{a} {b}");
                assert_eq!(x.ct_le(&y).unwrap_u8() == 1, a <= b, "{a} {b}");
                assert_eq!(x.ct_ge(&y).unwrap_u8() == 1, a >= b, "{a} {b}");
            }
        }
    }

    #[test]
    fn a_narrower_operand_is_sign_extended_before_ordering() {
        assert!(PaddedBigInt::from(-1i8) < PaddedBigInt::from(1i128));
        assert!(PaddedBigInt::from(-1i128) < PaddedBigInt::from(0i8));
        assert!(PaddedBigInt::from(-1i8) > PaddedBigInt::from(-2i128));
        let two_fifty_five = PaddedBigInt::from_le(&[0xffu8, 0x00]).unwrap();
        assert!(PaddedBigInt::from(-1i8) < two_fifty_five);
    }
}
