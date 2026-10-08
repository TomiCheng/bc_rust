//! The multiplicative identity of [`FixedBigUint`].

use num_traits::{ConstOne, One};

use super::FixedBigUint;
use crate::{Limb, LimbArray};

/// One in the low limb, built through `From<u8>`, so that it fails to
/// compile without limbs. `is_one` goes through `==`, so every method is
/// constant time.
impl<const N: usize> One for FixedBigUint<N> {
    fn one() -> Self {
        Self::from(1u8)
    }
}

/// One in the low limb, in a const, which fails to build without limbs, as
/// `one` fails to compile.
impl<const N: usize> ConstOne for FixedBigUint<N> {
    const ONE: Self = {
        assert!(N > 0, "no limbs to hold one");
        let mut limbs = [Limb::new(0); N];
        limbs[0] = Limb::new(1);
        Self::new(LimbArray::new(limbs))
    };
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

    #[test]
    fn the_constant_one_is_one() {
        use num_traits::ConstOne;

        const ONE: FixedBigUint<2> = FixedBigUint::ONE;
        assert!(ONE.is_one());
        assert_eq!(FixedBigUint::<1>::ONE, FixedBigUint::from(1u8));
    }
}
