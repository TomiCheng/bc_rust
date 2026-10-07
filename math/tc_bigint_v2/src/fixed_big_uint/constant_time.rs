//! Constant-time comparison of [`FixedBigUint`].

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

use tc_constant_time::{Choice, ConstantTimeEq, ConstantTimeOrd};

use super::FixedBigUint;
use crate::limb::{ct_eq_extended, ct_lt_extended};

/// Constant time.
impl<const N: usize> ConstantTimeEq for FixedBigUint<N> {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        ct_eq_extended(self.as_limbs(), 0, rhs.as_limbs(), 0)
    }
}

/// Goes through [`ConstantTimeEq::ct_eq`], so `==` is constant time.
impl<const N: usize> PartialEq for FixedBigUint<N> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl<const N: usize> Eq for FixedBigUint<N> {}

/// Hashes every limb, which agrees with `==` as both sides have `N` limbs.
impl<const N: usize> Hash for FixedBigUint<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_limbs().hash(state);
    }
}

/// Constant time.
impl<const N: usize> ConstantTimeOrd for FixedBigUint<N> {
    fn ct_lt(&self, rhs: &Self) -> Choice {
        ct_lt_extended(self.as_limbs(), 0, rhs.as_limbs(), 0, false)
    }
}

/// Goes through [`ConstantTimeOrd::ct_lt`] and `ct_eq`, so it is constant time.
impl<const N: usize> PartialOrd for FixedBigUint<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Goes through [`ConstantTimeOrd::ct_lt`] and `ct_eq`, so it is constant time.
impl<const N: usize> Ord for FixedBigUint<N> {
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

    use super::FixedBigUint;
    use crate::{Limb, LimbArray};

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
        assert!(equal(
            &FixedBigUint::<4>::from(7u8),
            &FixedBigUint::<4>::from(7u64)
        ));
        assert!(!equal(
            &FixedBigUint::<4>::from(7u8),
            &FixedBigUint::<4>::from(8u8)
        ));
    }

    #[test]
    fn a_difference_in_the_top_limb_is_found() {
        let low = FixedBigUint::<4>::new(LimbArray::new([Limb::new(1); 4]));
        let mut limbs = [Limb::new(1); 4];
        limbs[3] = Limb::new(2);
        assert!(!equal(&low, &FixedBigUint::<4>::new(LimbArray::new(limbs))));
    }

    #[test]
    fn ordering_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                assert_eq!(
                    FixedBigUint::<8>::from(a).cmp(&FixedBigUint::<8>::from(b)),
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
                let (x, y) = (FixedBigUint::<8>::from(a), FixedBigUint::<8>::from(b));
                assert_eq!(x.ct_lt(&y).unwrap_u8() == 1, a < b, "{a} {b}");
                assert_eq!(x.ct_gt(&y).unwrap_u8() == 1, a > b, "{a} {b}");
                assert_eq!(x.ct_le(&y).unwrap_u8() == 1, a <= b, "{a} {b}");
                assert_eq!(x.ct_ge(&y).unwrap_u8() == 1, a >= b, "{a} {b}");
            }
        }
    }

    #[test]
    fn the_highest_differing_limb_decides() {
        let high = FixedBigUint::<2>::new(LimbArray::new([Limb::new(5), Limb::new(1)]));
        let low = FixedBigUint::<2>::new(LimbArray::new([Limb::new(9), Limb::new(0)]));
        assert!(high > low);
    }
}
