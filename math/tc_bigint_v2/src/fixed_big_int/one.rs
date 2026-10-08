//! The multiplicative identity of [`FixedBigInt`].

use num_traits::{ConstOne, One};

use super::FixedBigInt;
use crate::{Limb, LimbArray};

/// One in the low limb, built through `From<i8>`, so that it fails to
/// compile without limbs. `is_one` goes through `==`, so every method is
/// constant time.
impl<const N: usize> One for FixedBigInt<N> {
    fn one() -> Self {
        Self::from(1i8)
    }
}

/// One in the low limb, in a const, which fails to build without limbs, as
/// `one` fails to compile.
impl<const N: usize> ConstOne for FixedBigInt<N> {
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

    use crate::{FixedBigInt, Limb, LimbArray};

    #[test]
    fn only_one_is_one() {
        assert!(FixedBigInt::<2>::one().is_one());
        assert!(!FixedBigInt::<2>::zero().is_one());
        assert!(!FixedBigInt::<2>::from(-1i8).is_one());
        let high = FixedBigInt::<2>::new(LimbArray::new([Limb::new(1), Limb::new(1)]));
        assert!(!high.is_one());
    }

    #[test]
    fn multiplying_by_one_leaves_a_value_unchanged() {
        let a = FixedBigInt::<2>::from(-255i16);
        assert_eq!(&a * &FixedBigInt::one(), a);
        assert_eq!(&FixedBigInt::one() * &a, a);
    }

    #[test]
    fn setting_one_makes_a_value_one() {
        let mut a = FixedBigInt::<2>::from(-255i16);
        a.set_one();
        assert!(a.is_one());
    }

    #[test]
    fn the_constant_one_is_one() {
        use num_traits::ConstOne;

        const ONE: FixedBigInt<2> = FixedBigInt::ONE;
        assert!(ONE.is_one());
        assert_eq!(FixedBigInt::<1>::ONE, FixedBigInt::from(1i8));
    }
}
