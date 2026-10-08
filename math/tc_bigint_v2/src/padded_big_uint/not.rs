//! Bitwise complement of [`PaddedBigUint`].

use core::ops::Not;

use super::PaddedBigUint;
use crate::limb::invert;

/// Every bit of the width inverted, in place, so that the result follows
/// the width: equal values of different widths give different results, as
/// `!1u8` and `!1u128` do. Constant time.
impl Not for PaddedBigUint {
    type Output = PaddedBigUint;

    fn not(mut self) -> PaddedBigUint {
        invert(self.limbs_mut());
        self
    }
}

/// Inverts a copy of `self`, so it allocates once. Constant time.
impl Not for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn not(self) -> PaddedBigUint {
        !self.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Limb, PaddedBigUint, Word};

    const VALUES: [u128; 7] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX,
    ];

    #[test]
    fn inverting_matches_the_primitive_one() {
        for a in VALUES {
            let x = PaddedBigUint::from(a);
            assert_eq!(!&x, PaddedBigUint::from(!a), "{a}");
            assert_eq!(!!x.clone(), x, "{a}");
        }
    }

    #[test]
    fn both_forms_give_the_same_result() {
        let x = PaddedBigUint::from(255u8);
        assert_eq!(!x.clone(), !&x);
    }

    #[test]
    fn the_result_follows_the_width() {
        let narrow = !PaddedBigUint::from(1u8);
        assert_eq!(narrow.as_limbs(), [Limb::new(Word::MAX - 1)]);
        let wide = !PaddedBigUint::from(1u128);
        assert_eq!(wide.as_limbs().len(), (u128::BITS / Word::BITS) as usize);
        assert_ne!(narrow, wide);
    }
}
