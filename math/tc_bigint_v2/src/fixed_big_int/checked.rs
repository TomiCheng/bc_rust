//! Checked arithmetic of [`FixedBigInt`]: `None` where the operators panic.

use num_traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedShl, CheckedShr, CheckedSub, Zero,
};

use super::FixedBigInt;
use crate::Word;

/// `None` on overflow. Variable time: only for public values, as the result
/// shows whether the sum overflowed; the sum itself is worked out in
/// constant time.
impl<const N: usize> CheckedAdd for FixedBigInt<N> {
    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        let mut sum = self.clone();
        (!sum.overflowing_add_assign(rhs)).then_some(sum)
    }
}

/// `None` on overflow. Variable time: only for public values, as the result
/// shows whether the difference overflowed; the difference itself is worked
/// out in constant time.
impl<const N: usize> CheckedSub for FixedBigInt<N> {
    fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        let mut difference = self.clone();
        (!difference.overflowing_sub_assign(rhs)).then_some(difference)
    }
}

/// `None` on overflow. Variable time: only for public values, as the result
/// shows whether the product overflowed; the product itself is worked out
/// in constant time.
impl<const N: usize> CheckedMul for FixedBigInt<N> {
    fn checked_mul(&self, rhs: &Self) -> Option<Self> {
        let mut product = self.clone();
        (!product.overflowing_mul_assign(rhs)).then_some(product)
    }
}

/// `None` when `rhs` is zero, or when the quotient does not fit, which only
/// the most negative value divided by -1 does. Variable time: only for
/// public values, as the result shows whether either happened; the quotient
/// itself is worked out in constant time.
impl<const N: usize> CheckedDiv for FixedBigInt<N> {
    fn checked_div(&self, rhs: &Self) -> Option<Self> {
        if rhs.is_zero() {
            return None;
        }
        let mut quotient = self.clone();
        let (_, overflowed) = quotient.divide(rhs);
        (!overflowed).then_some(quotient)
    }
}

/// `None` when `rhs` is zero; the most negative value by -1 gives zero, as
/// `%` does, where the primitive integers give `None`. Variable time: only
/// for public values, as the result shows whether `rhs` is zero; the
/// remainder itself is worked out in constant time.
impl<const N: usize> CheckedRem for FixedBigInt<N> {
    fn checked_rem(&self, rhs: &Self) -> Option<Self> {
        if rhs.is_zero() {
            return None;
        }
        let mut quotient = self.clone();
        Some(quotient.divide(rhs).0)
    }
}

/// `None` when `shift` is not below the width. Constant time in the value;
/// `shift` must be public.
impl<const N: usize> CheckedShl for FixedBigInt<N> {
    fn checked_shl(&self, shift: u32) -> Option<Self> {
        ((shift as usize) < N * Word::BITS as usize).then(|| self << shift)
    }
}

/// `None` when `shift` is not below the width. Constant time in the value;
/// `shift` must be public.
impl<const N: usize> CheckedShr for FixedBigInt<N> {
    fn checked_shr(&self, shift: u32) -> Option<Self> {
        ((shift as usize) < N * Word::BITS as usize).then(|| self >> shift)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{
        CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedShl, CheckedShr, CheckedSub,
    };

    use crate::{FixedBigInt, Word};

    /// The limbs of 128 bits, to compare against `i128`.
    const LIMBS: usize = (i128::BITS / Word::BITS) as usize;

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

    const SHIFTS: [u32; 6] = [0, 1, Word::BITS, 127, 128, 200];

    #[test]
    fn checked_arithmetic_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (FixedBigInt::<LIMBS>::from(a), FixedBigInt::<LIMBS>::from(b));
                assert_eq!(
                    x.checked_add(&y),
                    a.checked_add(b).map(FixedBigInt::<LIMBS>::from),
                    "{a} {b}"
                );
                assert_eq!(
                    x.checked_sub(&y),
                    a.checked_sub(b).map(FixedBigInt::<LIMBS>::from),
                    "{a} {b}"
                );
                assert_eq!(
                    x.checked_mul(&y),
                    a.checked_mul(b).map(FixedBigInt::<LIMBS>::from),
                    "{a} {b}"
                );
                assert_eq!(
                    x.checked_div(&y),
                    a.checked_div(b).map(FixedBigInt::<LIMBS>::from),
                    "{a} {b}"
                );
                // the most negative value by -1: zero here, None for i128
                if (a, b) == (i128::MIN, -1) {
                    assert_eq!(x.checked_rem(&y), Some(FixedBigInt::<LIMBS>::from(0i8)));
                    continue;
                }
                assert_eq!(
                    x.checked_rem(&y),
                    a.checked_rem(b).map(FixedBigInt::<LIMBS>::from),
                    "{a} {b}"
                );
            }
        }
    }

    #[test]
    fn checked_shifts_match_the_primitive_ones() {
        for a in VALUES {
            for shift in SHIFTS {
                let x = FixedBigInt::<LIMBS>::from(a);
                assert_eq!(
                    x.checked_shl(shift),
                    a.checked_shl(shift).map(FixedBigInt::<LIMBS>::from),
                    "{a} {shift}"
                );
                assert_eq!(
                    x.checked_shr(shift),
                    a.checked_shr(shift).map(FixedBigInt::<LIMBS>::from),
                    "{a} {shift}"
                );
            }
        }
    }
}
