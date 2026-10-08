//! The additive identity of [`FixedBigInt`].

use num_traits::Zero;

use super::FixedBigInt;

/// Zero in all `N` limbs. `is_zero` goes through `==`, so every method is
/// constant time.
impl<const N: usize> Zero for FixedBigInt<N> {
    fn zero() -> Self {
        Self::default()
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;

    use crate::{FixedBigInt, Limb, LimbArray};

    #[test]
    fn only_zero_is_zero() {
        assert!(FixedBigInt::<2>::zero().is_zero());
        assert!(!FixedBigInt::<2>::from(1i8).is_zero());
        assert!(!FixedBigInt::<2>::from(-1i8).is_zero());
        let top = FixedBigInt::<2>::new(LimbArray::new([Limb::new(0), Limb::new(1)]));
        assert!(!top.is_zero());
    }

    #[test]
    fn adding_zero_leaves_a_value_unchanged() {
        let a = FixedBigInt::<2>::from(-255i16);
        assert_eq!(&a + &FixedBigInt::zero(), a);
        assert_eq!(&FixedBigInt::zero() + &a, a);
    }

    #[test]
    fn setting_zero_makes_a_value_zero() {
        let mut a = FixedBigInt::<2>::from(-255i16);
        a.set_zero();
        assert!(a.is_zero());
    }
}
