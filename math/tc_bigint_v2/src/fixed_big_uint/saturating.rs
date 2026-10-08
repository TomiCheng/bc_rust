//! Saturating arithmetic of [`FixedBigUint`].

use num_traits::{SaturatingAdd, SaturatingMul, SaturatingSub};

use super::FixedBigUint;
use crate::Word;
use crate::limb::saturate_unsigned;

/// The largest value in place of an overflow. Constant time.
impl<const N: usize> SaturatingAdd for FixedBigUint<N> {
    fn saturating_add(&self, rhs: &Self) -> Self {
        let mut sum = self.clone();
        let overflowed = sum.overflowing_add_assign(rhs);
        saturate_unsigned(sum.limbs_mut(), overflowed, Word::MAX);
        sum
    }
}

/// Zero in place of a negative difference. Constant time.
impl<const N: usize> SaturatingSub for FixedBigUint<N> {
    fn saturating_sub(&self, rhs: &Self) -> Self {
        let mut difference = self.clone();
        let overflowed = difference.overflowing_sub_assign(rhs);
        saturate_unsigned(difference.limbs_mut(), overflowed, 0);
        difference
    }
}

/// The largest value in place of an overflow. Constant time.
impl<const N: usize> SaturatingMul for FixedBigUint<N> {
    fn saturating_mul(&self, rhs: &Self) -> Self {
        let mut product = self.clone();
        let overflowed = product.overflowing_mul_assign(rhs);
        saturate_unsigned(product.limbs_mut(), overflowed, Word::MAX);
        product
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{SaturatingAdd, SaturatingMul, SaturatingSub};

    use crate::{FixedBigUint, Word};

    /// The limbs of 128 bits, to compare against `u128`.
    const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

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
    fn saturating_arithmetic_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (
                    FixedBigUint::<LIMBS>::from(a),
                    FixedBigUint::<LIMBS>::from(b),
                );
                assert_eq!(
                    x.saturating_add(&y),
                    FixedBigUint::<LIMBS>::from(a.saturating_add(b)),
                    "{a} {b}"
                );
                assert_eq!(
                    x.saturating_sub(&y),
                    FixedBigUint::<LIMBS>::from(a.saturating_sub(b)),
                    "{a} {b}"
                );
                assert_eq!(
                    x.saturating_mul(&y),
                    FixedBigUint::<LIMBS>::from(a.saturating_mul(b)),
                    "{a} {b}"
                );
            }
        }
    }
}
