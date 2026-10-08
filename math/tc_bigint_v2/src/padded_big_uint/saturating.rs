//! Saturating arithmetic of [`PaddedBigUint`].

use num_traits::{SaturatingAdd, SaturatingMul, SaturatingSub};

use super::PaddedBigUint;
use crate::Word;
use crate::limb::saturate_unsigned;

/// The largest value at the wider width in place of an overflow. Constant
/// time.
impl SaturatingAdd for PaddedBigUint {
    fn saturating_add(&self, rhs: &Self) -> Self {
        let mut sum = self.clone_for(rhs);
        let overflowed = sum.overflowing_add_assign(rhs);
        saturate_unsigned(sum.limbs_mut(), overflowed, Word::MAX);
        sum
    }
}

/// Zero at the wider width in place of a negative difference. Constant
/// time.
impl SaturatingSub for PaddedBigUint {
    fn saturating_sub(&self, rhs: &Self) -> Self {
        let mut difference = self.clone_for(rhs);
        let overflowed = difference.overflowing_sub_assign(rhs);
        saturate_unsigned(difference.limbs_mut(), overflowed, 0);
        difference
    }
}

/// The largest value at the wider width in place of an overflow. Constant
/// time.
impl SaturatingMul for PaddedBigUint {
    fn saturating_mul(&self, rhs: &Self) -> Self {
        let mut product = self.clone_for(rhs);
        let overflowed = product.overflowing_mul_assign(rhs);
        saturate_unsigned(product.limbs_mut(), overflowed, Word::MAX);
        product
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{SaturatingAdd, SaturatingMul, SaturatingSub};

    use crate::PaddedBigUint;

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
                let (x, y) = (PaddedBigUint::from(a), PaddedBigUint::from(b));
                assert_eq!(
                    x.saturating_add(&y),
                    PaddedBigUint::from(a.saturating_add(b)),
                    "{a} {b}"
                );
                assert_eq!(
                    x.saturating_sub(&y),
                    PaddedBigUint::from(a.saturating_sub(b)),
                    "{a} {b}"
                );
                assert_eq!(
                    x.saturating_mul(&y),
                    PaddedBigUint::from(a.saturating_mul(b)),
                    "{a} {b}"
                );
            }
        }
    }
}
