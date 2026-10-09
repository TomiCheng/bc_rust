//! The additive identity of [`BigUint`].

use num_traits::{ConstZero, Zero};

use super::BigUint;

/// Zero has no limbs, so `is_zero` only checks for none, and `set_zero`
/// keeps the storage for later use. Constant time.
impl Zero for BigUint {
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

/// No limbs, in a const; there is no `ConstOne`, as one needs a limb, which a
/// const cannot allocate. Constant time.
impl ConstZero for BigUint {
    const ZERO: Self = Self::empty();
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;

    use crate::BigUint;

    #[test]
    fn zero_has_no_limbs() {
        assert!(BigUint::zero().as_limbs().is_empty());
    }

    #[test]
    fn only_zero_is_zero() {
        assert!(BigUint::zero().is_zero());
        assert!(BigUint::from(0u128).is_zero());
        assert!(!BigUint::from(1u8).is_zero());
    }

    #[test]
    fn adding_zero_leaves_a_value_unchanged() {
        let a = BigUint::from(u128::MAX);
        assert_eq!(&a + &BigUint::zero(), a);
        assert_eq!(&BigUint::zero() + &a, a);
    }

    #[test]
    fn setting_zero_leaves_no_limbs() {
        let mut a = BigUint::from(u128::MAX);
        a.set_zero();
        assert!(a.as_limbs().is_empty());
    }

    #[test]
    fn the_constant_zero_has_no_limbs() {
        use num_traits::ConstZero;

        const ZERO: BigUint = BigUint::ZERO;
        assert!(ZERO.as_limbs().is_empty());
        assert_eq!(ZERO, BigUint::zero());
    }
}
