//! The multiplicative identity of [`PaddedBigUint`].

use num_traits::One;

use super::PaddedBigUint;
use crate::Limb;
use crate::limb::ct_eq_extended;

/// One in a single limb, as `From<u8>` gives: a product takes the wider
/// width, so multiplying by it leaves a value as it is, width included,
/// unless the width is zero. `is_one` holds at any width without building
/// one, and `set_one` keeps the width and the storage, except that a value
/// of width zero takes one limb. Constant time.
impl One for PaddedBigUint {
    fn one() -> Self {
        Self::from(1u8)
    }

    fn is_one(&self) -> bool {
        ct_eq_extended(self.as_limbs(), 0, &[Limb::new(1)], 0).unwrap_u8() == 1
    }

    fn set_one(&mut self) {
        match self.limbs_mut() {
            [] => *self = Self::one(),
            [low, rest @ ..] => {
                *low = Limb::new(1);
                rest.fill(Limb::new(0));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{One, Zero};

    use crate::{PaddedBigUint, Word};

    #[test]
    fn one_has_a_single_limb() {
        assert_eq!(PaddedBigUint::one().as_limbs().len(), 1);
    }

    #[test]
    fn only_one_is_one_at_any_width() {
        assert!(PaddedBigUint::one().is_one());
        assert!(PaddedBigUint::from(1u128).is_one());
        assert!(!PaddedBigUint::zero().is_one());
        assert!(!PaddedBigUint::from(2u8).is_one());
        assert!(!PaddedBigUint::from(1u128 << 64 | 1).is_one());
    }

    #[test]
    fn multiplying_by_one_leaves_a_value_and_its_width_unchanged() {
        let a = PaddedBigUint::from(255u128);
        for product in [&a * &PaddedBigUint::one(), &PaddedBigUint::one() * &a] {
            assert_eq!(product, a);
            assert_eq!(product.as_limbs().len(), a.as_limbs().len());
        }
    }

    #[test]
    fn setting_one_keeps_the_width() {
        let mut a = PaddedBigUint::from(255u128);
        a.set_one();
        assert!(a.is_one());
        assert_eq!(a.as_limbs().len(), (u128::BITS / Word::BITS) as usize);
    }

    #[test]
    fn setting_one_on_width_zero_takes_one_limb() {
        let mut a = PaddedBigUint::zero();
        a.set_one();
        assert!(a.is_one());
        assert_eq!(a.as_limbs().len(), 1);
    }
}
