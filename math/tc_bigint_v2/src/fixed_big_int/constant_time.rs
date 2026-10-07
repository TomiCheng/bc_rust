//! Constant-time comparison of [`FixedBigInt`].

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

use tc_constant_time::{Choice, ConstantTimeEq, ConstantTimeOrd};

use super::FixedBigInt;
use crate::encoding::sign_fill;
use crate::limb::{ct_eq_extended, ct_lt_extended};

/// Constant time.
impl<const N: usize> ConstantTimeEq for FixedBigInt<N> {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        let (left, right) = (self.as_limbs(), rhs.as_limbs());
        ct_eq_extended(left, sign_fill(left), right, sign_fill(right))
    }
}

/// Goes through [`ConstantTimeEq::ct_eq`], so `==` is constant time.
impl<const N: usize> PartialEq for FixedBigInt<N> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl<const N: usize> Eq for FixedBigInt<N> {}

/// Hashes every limb, which agrees with `==` as both sides have `N` limbs.
impl<const N: usize> Hash for FixedBigInt<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_limbs().hash(state);
    }
}

/// Constant time.
impl<const N: usize> ConstantTimeOrd for FixedBigInt<N> {
    fn ct_lt(&self, rhs: &Self) -> Choice {
        let (left, right) = (self.as_limbs(), rhs.as_limbs());
        ct_lt_extended(left, sign_fill(left), right, sign_fill(right), true)
    }
}

/// Goes through [`ConstantTimeOrd::ct_lt`] and `ct_eq`, so it is constant time.
impl<const N: usize> PartialOrd for FixedBigInt<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Goes through [`ConstantTimeOrd::ct_lt`] and `ct_eq`, so it is constant time.
impl<const N: usize> Ord for FixedBigInt<N> {
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

    use super::FixedBigInt;
    use crate::{Limb, LimbArray, Word};

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
    fn equal_values_compare_equal_and_the_sign_matters() {
        assert!(equal(
            &FixedBigInt::<4>::from(-7i8),
            &FixedBigInt::<4>::from(-7i64)
        ));
        assert!(!equal(
            &FixedBigInt::<4>::from(-7i8),
            &FixedBigInt::<4>::from(7i8)
        ));
    }

    #[test]
    fn a_difference_in_the_sign_limb_is_found() {
        let minus_one = FixedBigInt::<2>::from(-1i8);
        let mut limbs = [Limb::new(Word::MAX); 2];
        limbs[1] = Limb::new(0);
        assert!(!equal(
            &minus_one,
            &FixedBigInt::<2>::new(LimbArray::new(limbs))
        ));
    }

    #[test]
    fn ordering_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                assert_eq!(
                    FixedBigInt::<8>::from(a).cmp(&FixedBigInt::<8>::from(b)),
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
                let (x, y) = (FixedBigInt::<8>::from(a), FixedBigInt::<8>::from(b));
                assert_eq!(x.ct_lt(&y).unwrap_u8() == 1, a < b, "{a} {b}");
                assert_eq!(x.ct_gt(&y).unwrap_u8() == 1, a > b, "{a} {b}");
                assert_eq!(x.ct_le(&y).unwrap_u8() == 1, a <= b, "{a} {b}");
                assert_eq!(x.ct_ge(&y).unwrap_u8() == 1, a >= b, "{a} {b}");
            }
        }
    }

    #[test]
    fn a_set_sign_bit_sorts_below_every_positive_value() {
        let top = Limb::new(1 << (Word::BITS - 1));
        let most_negative = FixedBigInt::<1>::new(LimbArray::new([top]));
        assert!(most_negative < FixedBigInt::<1>::from(0i8));
        assert!(most_negative < FixedBigInt::<1>::from(-1i8));
    }
}
