//! Wrapping arithmetic of [`FixedBigUint`].

use num_traits::{WrappingAdd, WrappingMul, WrappingSub};

use super::FixedBigUint;

/// The sum wrapped around at the `N` limbs, as the primitive integers'
/// `wrapping_add` gives it. Constant time.
impl<const N: usize> WrappingAdd for FixedBigUint<N> {
    fn wrapping_add(&self, rhs: &Self) -> Self {
        let mut sum = self.clone();
        // the wrapped sum is all that is wanted, not whether it overflowed
        sum.overflowing_add_assign(rhs);
        sum
    }
}

/// The difference wrapped around at the `N` limbs, as the primitive
/// integers' `wrapping_sub` gives it. Constant time.
impl<const N: usize> WrappingSub for FixedBigUint<N> {
    fn wrapping_sub(&self, rhs: &Self) -> Self {
        let mut difference = self.clone();
        // the wrapped difference is all that is wanted, not whether it overflowed
        difference.overflowing_sub_assign(rhs);
        difference
    }
}

/// The product wrapped around at the `N` limbs, as the primitive integers'
/// `wrapping_mul` gives it. Constant time.
impl<const N: usize> WrappingMul for FixedBigUint<N> {
    fn wrapping_mul(&self, rhs: &Self) -> Self {
        let mut product = self.clone();
        // the wrapped product is all that is wanted, not whether it overflowed
        product.overflowing_mul_assign(rhs);
        product
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{WrappingAdd, WrappingMul, WrappingSub};

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
    fn wrapping_arithmetic_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (
                    FixedBigUint::<LIMBS>::from(a),
                    FixedBigUint::<LIMBS>::from(b),
                );
                assert_eq!(
                    x.wrapping_add(&y),
                    FixedBigUint::<LIMBS>::from(a.wrapping_add(b)),
                    "{a} {b}"
                );
                assert_eq!(
                    x.wrapping_sub(&y),
                    FixedBigUint::<LIMBS>::from(a.wrapping_sub(b)),
                    "{a} {b}"
                );
                assert_eq!(
                    x.wrapping_mul(&y),
                    FixedBigUint::<LIMBS>::from(a.wrapping_mul(b)),
                    "{a} {b}"
                );
            }
        }
    }
}
