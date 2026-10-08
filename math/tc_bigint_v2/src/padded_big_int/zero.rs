//! The additive identity of [`PaddedBigInt`].

use num_traits::Zero;

use super::PaddedBigInt;
use crate::Limb;

/// Zero of width zero, as `Default` gives: a sum takes the wider width, so
/// adding it leaves a value as it is, width included. `is_zero` holds at
/// any width, through `==`, and `set_zero` keeps the width and the storage.
/// Constant time.
impl Zero for PaddedBigInt {
    fn zero() -> Self {
        Self::default()
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    fn set_zero(&mut self) {
        self.limbs_mut().fill(Limb::new(0));
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;

    use crate::{PaddedBigInt, Word};

    #[test]
    fn zero_has_width_zero() {
        assert!(PaddedBigInt::zero().as_limbs().is_empty());
    }

    #[test]
    fn only_zero_is_zero_at_any_width() {
        assert!(PaddedBigInt::zero().is_zero());
        assert!(PaddedBigInt::from(0i128).is_zero());
        assert!(!PaddedBigInt::from(1i8).is_zero());
        assert!(!PaddedBigInt::from(-1i8).is_zero());
        assert!(!PaddedBigInt::from(i128::MIN).is_zero());
    }

    #[test]
    fn adding_zero_leaves_a_value_and_its_width_unchanged() {
        let a = PaddedBigInt::from(-255i128);
        for sum in [&a + &PaddedBigInt::zero(), &PaddedBigInt::zero() + &a] {
            assert_eq!(sum, a);
            assert_eq!(sum.as_limbs().len(), a.as_limbs().len());
        }
    }

    #[test]
    fn setting_zero_keeps_the_width() {
        let mut a = PaddedBigInt::from(-255i128);
        a.set_zero();
        assert!(a.is_zero());
        assert_eq!(a.as_limbs().len(), (i128::BITS / Word::BITS) as usize);
    }
}
