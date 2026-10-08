//! The multiplicative identity of [`FixedBigUint`].

use num_traits::One;

use super::FixedBigUint;

/// One in the low limb, built through `From<u8>`, so that it fails to
/// compile without limbs. `is_one` goes through `==`, so every method is
/// constant time.
impl<const N: usize> One for FixedBigUint<N> {
    fn one() -> Self {
        Self::from(1u8)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{One, Zero};

    use crate::{FixedBigUint, Limb, LimbArray};

    #[test]
    fn only_one_is_one() {
        assert!(FixedBigUint::<2>::one().is_one());
        assert!(!FixedBigUint::<2>::zero().is_one());
        assert!(!FixedBigUint::<2>::from(2u8).is_one());
        let high = FixedBigUint::<2>::new(LimbArray::new([Limb::new(1), Limb::new(1)]));
        assert!(!high.is_one());
    }

    #[test]
    fn multiplying_by_one_leaves_a_value_unchanged() {
        let a = FixedBigUint::<2>::from(255u8);
        assert_eq!(&a * &FixedBigUint::one(), a);
        assert_eq!(&FixedBigUint::one() * &a, a);
    }

    #[test]
    fn setting_one_makes_a_value_one() {
        let mut a = FixedBigUint::<2>::from(255u8);
        a.set_one();
        assert!(a.is_one());
    }
}
