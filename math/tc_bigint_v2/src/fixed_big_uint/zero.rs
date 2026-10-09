//! The additive identity of [`FixedBigUint`].

use num_traits::{ConstZero, Zero};

use super::FixedBigUint;
use crate::{Limb, LimbArray};

/// Zero in all `N` limbs. `is_zero` goes through `==`. Constant time.
impl<const N: usize> Zero for FixedBigUint<N> {
    fn zero() -> Self {
        Self::default()
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

/// Zero in all `N` limbs, in a const. Constant time.
impl<const N: usize> ConstZero for FixedBigUint<N> {
    const ZERO: Self = Self::new(LimbArray::new([Limb::new(0); N]));
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;

    use crate::{FixedBigUint, Limb, LimbArray};

    #[test]
    fn only_zero_is_zero() {
        assert!(FixedBigUint::<2>::zero().is_zero());
        assert!(!FixedBigUint::<2>::from(1u8).is_zero());
        let top = FixedBigUint::<2>::new(LimbArray::new([Limb::new(0), Limb::new(1)]));
        assert!(!top.is_zero());
    }

    #[test]
    fn adding_zero_leaves_a_value_unchanged() {
        let a = FixedBigUint::<2>::from(255u8);
        assert_eq!(&a + &FixedBigUint::zero(), a);
        assert_eq!(&FixedBigUint::zero() + &a, a);
    }

    #[test]
    fn setting_zero_makes_a_value_zero() {
        let mut a = FixedBigUint::<2>::from(255u8);
        a.set_zero();
        assert!(a.is_zero());
    }

    #[test]
    fn the_constant_zero_is_zero() {
        use num_traits::ConstZero;

        const ZERO: FixedBigUint<2> = FixedBigUint::ZERO;
        assert!(ZERO.is_zero());
    }
}
