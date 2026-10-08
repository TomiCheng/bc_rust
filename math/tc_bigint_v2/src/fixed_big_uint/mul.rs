//! Multiplication of [`FixedBigUint`].

use core::ops::{Mul, MulAssign};

use super::FixedBigUint;
use crate::limb::mul_assign_limbs;

/// In place, schoolbook over the `N` limbs; panics on overflow in every
/// build, unlike the primitive integers, whose check depends on the
/// profile. Constant time, apart from that panic.
impl<const N: usize> MulAssign<&FixedBigUint<N>> for FixedBigUint<N> {
    fn mul_assign(&mut self, rhs: &FixedBigUint<N>) {
        let lost = mul_assign_limbs(self.limbs_mut(), rhs.as_limbs(), 0);
        assert!(!lost, "attempt to multiply with overflow");
    }
}

/// The same as `*= &rhs`. Constant time, apart from the panic on overflow.
impl<const N: usize> MulAssign<FixedBigUint<N>> for FixedBigUint<N> {
    fn mul_assign(&mut self, rhs: FixedBigUint<N>) {
        *self *= &rhs;
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Mul<&FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn mul(mut self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self *= rhs;
        self
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Mul<FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn mul(mut self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        self *= &rhs;
        self
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Constant
/// time, apart from the panic on overflow.
impl<const N: usize> Mul<FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn mul(self, mut rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        rhs *= self;
        rhs
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Mul<&FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn mul(self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self.clone() * rhs
    }
}

#[cfg(test)]
mod tests {
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
    fn products_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(product) = a.checked_mul(b) {
                    assert_eq!(
                        &FixedBigUint::<LIMBS>::from(a) * &FixedBigUint::<LIMBS>::from(b),
                        FixedBigUint::<LIMBS>::from(product),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            FixedBigUint::<LIMBS>::from(255u8),
            FixedBigUint::<LIMBS>::from(0xf0f0u16),
        );
        let expected = FixedBigUint::<LIMBS>::from(255u32 * 0xf0f0);
        assert_eq!(x.clone() * y.clone(), expected);
        assert_eq!(x.clone() * &y, expected);
        assert_eq!(&x * y.clone(), expected);
        assert_eq!(&x * &y, expected);
        let mut owned = x.clone();
        owned *= y.clone();
        assert_eq!(owned, expected);
        let mut borrowed = x;
        borrowed *= &y;
        assert_eq!(borrowed, expected);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn a_product_past_the_width_panics() {
        let _ = FixedBigUint::<LIMBS>::from(u128::MAX) * FixedBigUint::<LIMBS>::from(2u8);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn a_product_whose_low_half_is_zero_still_panics() {
        let half = FixedBigUint::<LIMBS>::from(1u128 << 64);
        let _ = &half * &half;
    }
}
