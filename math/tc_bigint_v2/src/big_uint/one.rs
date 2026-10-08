//! The multiplicative identity of [`BigUint`].

use num_traits::One;

use super::BigUint;
use crate::Limb;

/// One is a single limb, so `is_one` checks for just that limb without
/// building one, and `set_one` keeps the storage for later use. Constant
/// time.
impl One for BigUint {
    fn one() -> Self {
        Self::from(1u8)
    }

    fn is_one(&self) -> bool {
        self.as_limbs() == [Limb::new(1)]
    }

    fn set_one(&mut self) {
        let mut limbs = core::mem::take(self).into_limbs();
        limbs.clear();
        limbs.push(Limb::new(1));
        *self = Self::new(limbs);
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{One, Zero};

    use crate::BigUint;

    #[test]
    fn only_one_is_one() {
        assert!(BigUint::one().is_one());
        assert!(BigUint::from(1u128).is_one());
        assert!(!BigUint::zero().is_one());
        assert!(!BigUint::from(2u8).is_one());
        assert!(!BigUint::from(1u128 << 64 | 1).is_one());
    }

    #[test]
    fn multiplying_by_one_leaves_a_value_unchanged() {
        let a = BigUint::from(u128::MAX);
        assert_eq!(&a * &BigUint::one(), a);
        assert_eq!(&BigUint::one() * &a, a);
    }

    #[test]
    fn setting_one_makes_a_value_one() {
        for mut a in [BigUint::from(u128::MAX), BigUint::zero()] {
            a.set_one();
            assert!(a.is_one());
        }
    }
}
