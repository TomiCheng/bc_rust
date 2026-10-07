//! Constant-time comparison of [`FixedBigUint`].

use tc_constant_time::{Choice, ConstantTimeEq};

use super::FixedBigUint;
use crate::limb::ct_eq_extended;

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

#[cfg(test)]
mod tests {
    use tc_constant_time::ConstantTimeEq;

    use super::FixedBigUint;
    use crate::{Limb, LimbArray};

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
}
