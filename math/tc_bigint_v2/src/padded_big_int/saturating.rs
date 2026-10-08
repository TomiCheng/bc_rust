//! Saturating arithmetic of [`PaddedBigInt`].

use num_traits::{SaturatingAdd, SaturatingMul, SaturatingSub};

use super::PaddedBigInt;
use crate::encoding::sign_fill;
use crate::limb::saturate_signed;

/// The largest or the smallest value at the wider width in place of an
/// overflow, by the sign the sum would have had, which is that of both
/// operands. Constant time.
impl SaturatingAdd for PaddedBigInt {
    fn saturating_add(&self, rhs: &Self) -> Self {
        let sign = sign_fill(self.as_limbs());
        let mut sum = self.clone_for(rhs);
        let overflowed = sum.overflowing_add_assign(rhs);
        saturate_signed(sum.limbs_mut(), overflowed, sign);
        sum
    }
}

/// The largest or the smallest value at the wider width in place of an
/// overflow, by the sign the difference would have had, which is that of
/// `self`. Constant time.
impl SaturatingSub for PaddedBigInt {
    fn saturating_sub(&self, rhs: &Self) -> Self {
        let sign = sign_fill(self.as_limbs());
        let mut difference = self.clone_for(rhs);
        let overflowed = difference.overflowing_sub_assign(rhs);
        saturate_signed(difference.limbs_mut(), overflowed, sign);
        difference
    }
}

/// The largest or the smallest value at the wider width in place of an
/// overflow, by the sign the product would have had, which follows the
/// signs of both operands. Constant time.
impl SaturatingMul for PaddedBigInt {
    fn saturating_mul(&self, rhs: &Self) -> Self {
        let sign = sign_fill(self.as_limbs()) ^ sign_fill(rhs.as_limbs());
        let mut product = self.clone_for(rhs);
        let overflowed = product.overflowing_mul_assign(rhs);
        saturate_signed(product.limbs_mut(), overflowed, sign);
        product
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{SaturatingAdd, SaturatingMul, SaturatingSub};

    use crate::PaddedBigInt;

    const VALUES: [i128; 9] = [
        i128::MIN,
        i64::MIN as i128 - 1,
        -129,
        -1,
        0,
        1,
        255,
        u64::MAX as i128,
        i128::MAX,
    ];

    #[test]
    fn saturating_arithmetic_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (PaddedBigInt::from(a), PaddedBigInt::from(b));
                assert_eq!(
                    x.saturating_add(&y),
                    PaddedBigInt::from(a.saturating_add(b)),
                    "{a} {b}"
                );
                assert_eq!(
                    x.saturating_sub(&y),
                    PaddedBigInt::from(a.saturating_sub(b)),
                    "{a} {b}"
                );
                assert_eq!(
                    x.saturating_mul(&y),
                    PaddedBigInt::from(a.saturating_mul(b)),
                    "{a} {b}"
                );
            }
        }
    }
}
