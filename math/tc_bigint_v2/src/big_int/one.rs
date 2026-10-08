//! The multiplicative identity of [`BigInt`].

use num_traits::One;

use super::BigInt;
use crate::Limb;

/// One is a single limb, so `is_one` checks for just that limb without
/// building one, and `set_one` keeps the storage for later use. Constant
/// time.
impl One for BigInt {
    fn one() -> Self {
        Self::from(1i8)
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

    use crate::BigInt;

    #[test]
    fn only_one_is_one() {
        assert!(BigInt::one().is_one());
        assert!(BigInt::from(1i128).is_one());
        assert!(!BigInt::zero().is_one());
        assert!(!BigInt::from(-1i8).is_one());
        assert!(!BigInt::from(1i128 << 64 | 1).is_one());
    }

    #[test]
    fn multiplying_by_one_leaves_a_value_unchanged() {
        let a = BigInt::from(i128::MIN);
        assert_eq!(&a * &BigInt::one(), a);
        assert_eq!(&BigInt::one() * &a, a);
    }

    #[test]
    fn setting_one_makes_a_value_one() {
        for mut a in [BigInt::from(i128::MIN), BigInt::zero()] {
            a.set_one();
            assert!(a.is_one());
        }
    }
}
