//! Multiplication of [`FixedBigInt`].

use core::ops::{Mul, MulAssign};

use super::FixedBigInt;
use crate::limb::signed_mul_assign_limbs;

impl<const N: usize> FixedBigInt<N> {
    /// Multiplies by `rhs` in place, keeping the low limbs of the product, and
    /// returns whether that overflowed. Constant time.
    pub(super) fn overflowing_mul_assign(&mut self, rhs: &Self) -> bool {
        signed_mul_assign_limbs(self.limbs_mut(), rhs.as_limbs())
    }
}

/// In place, schoolbook over the `N` limbs on the magnitudes, then the
/// sign; panics on overflow in every build, unlike the primitive integers,
/// whose check depends on the profile. Constant time, apart from that
/// panic.
impl<const N: usize> MulAssign<&FixedBigInt<N>> for FixedBigInt<N> {
    fn mul_assign(&mut self, rhs: &FixedBigInt<N>) {
        let overflowed = self.overflowing_mul_assign(rhs);
        assert!(!overflowed, "attempt to multiply with overflow");
    }
}

/// The same as `*= &rhs`. Constant time, apart from the panic on overflow.
impl<const N: usize> MulAssign<FixedBigInt<N>> for FixedBigInt<N> {
    fn mul_assign(&mut self, rhs: FixedBigInt<N>) {
        *self *= &rhs;
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Mul<&FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn mul(mut self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self *= rhs;
        self
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Mul<FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn mul(mut self, rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        self *= &rhs;
        self
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Constant
/// time, apart from the panic on overflow.
impl<const N: usize> Mul<FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn mul(self, mut rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        rhs *= self;
        rhs
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Mul<&FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn mul(self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self.clone() * rhs
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn products_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(product) = a.checked_mul(b) {
                    assert_eq!(
                        &FixedBigInt::<LIMBS>::from(a) * &FixedBigInt::<LIMBS>::from(b),
                        FixedBigInt::<LIMBS>::from(product),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_most_negative_value_can_be_a_product() {
        let product = FixedBigInt::<LIMBS>::from(-(1i128 << 63)) * FixedBigInt::from(1i128 << 64);
        assert_eq!(product, FixedBigInt::from(i128::MIN));
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            FixedBigInt::<LIMBS>::from(-255i16),
            FixedBigInt::<LIMBS>::from(0xf0f0i32),
        );
        let expected = FixedBigInt::<LIMBS>::from(-255i32 * 0xf0f0);
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
    fn negating_the_most_negative_value_by_a_product_panics() {
        let _ = FixedBigInt::<LIMBS>::from(i128::MIN) * FixedBigInt::from(-1i8);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn a_product_past_the_largest_value_panics() {
        let _ = FixedBigInt::<LIMBS>::from(i128::MAX) * FixedBigInt::from(2i8);
    }
}
