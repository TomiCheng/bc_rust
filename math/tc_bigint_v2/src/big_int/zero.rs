//! The additive identity of [`BigInt`].

use num_traits::Zero;

use super::BigInt;

/// Zero has no limbs, so `is_zero` only checks for none, and `set_zero`
/// keeps the storage for later use. Constant time.
impl Zero for BigInt {
    fn zero() -> Self {
        Self::default()
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    fn set_zero(&mut self) {
        let mut limbs = core::mem::take(self).into_limbs();
        limbs.clear();
        *self = Self::new(limbs);
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;

    use crate::BigInt;

    #[test]
    fn zero_has_no_limbs() {
        assert!(BigInt::zero().as_limbs().is_empty());
    }

    #[test]
    fn only_zero_is_zero() {
        assert!(BigInt::zero().is_zero());
        assert!(BigInt::from(0i128).is_zero());
        assert!(!BigInt::from(1i8).is_zero());
        assert!(!BigInt::from(-1i8).is_zero());
    }

    #[test]
    fn adding_zero_leaves_a_value_unchanged() {
        let a = BigInt::from(i128::MIN);
        assert_eq!(&a + &BigInt::zero(), a);
        assert_eq!(&BigInt::zero() + &a, a);
    }

    #[test]
    fn setting_zero_leaves_no_limbs() {
        let mut a = BigInt::from(i128::MIN);
        a.set_zero();
        assert!(a.as_limbs().is_empty());
    }
}
