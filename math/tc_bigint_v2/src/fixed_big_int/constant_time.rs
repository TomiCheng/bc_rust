//! Constant-time comparison of [`FixedBigInt`].

use tc_constant_time::{Choice, ConstantTimeEq};

use super::FixedBigInt;
use crate::encoding::sign_fill;
use crate::limb::ct_eq_extended;

/// Constant time.
impl<const N: usize> ConstantTimeEq for FixedBigInt<N> {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        let (left, right) = (self.as_limbs(), rhs.as_limbs());
        ct_eq_extended(left, sign_fill(left), right, sign_fill(right))
    }
}

#[cfg(test)]
mod tests {
    use tc_constant_time::ConstantTimeEq;

    use super::FixedBigInt;
    use crate::{Limb, LimbArray, Word};

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
}
