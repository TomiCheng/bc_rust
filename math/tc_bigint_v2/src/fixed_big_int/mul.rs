//! Multiplication of [`FixedBigInt`].

use core::ops::{Mul, MulAssign};

use super::FixedBigInt;
use crate::Word;
use crate::encoding::sign_fill;
use crate::limb::{conditional_negate, ct_eq_extended, mul_assign_word, signed_mul_assign_limbs};

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

/// The magnitude multiplied by the word in one pass from the bottom, then
/// the sign put back, in place; panics on overflow in every build, unlike
/// the primitive integers, whose check depends on the profile. Constant
/// time, apart from that panic.
impl<const N: usize> MulAssign<u32> for FixedBigInt<N> {
    fn mul_assign(&mut self, rhs: u32) {
        let limbs = self.limbs_mut();
        let sign = sign_fill(limbs);
        conditional_negate(limbs, sign);
        let carry = mul_assign_word(limbs, Word::from(rhs));
        let nonzero = ct_eq_extended(limbs, 0, &[], 0).unwrap_u8() == 0;
        conditional_negate(limbs, sign);
        // a product that is not zero keeps the sign of `self`, or it did not
        // fit
        let overflowed = (carry != 0) | (nonzero & (sign_fill(limbs) != sign));
        assert!(!overflowed, "attempt to multiply with overflow");
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Mul<u32> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn mul(mut self, rhs: u32) -> FixedBigInt<N> {
        self *= rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Mul<u32> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn mul(self, rhs: u32) -> FixedBigInt<N> {
        self.clone() * rhs
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Constant time,
/// apart from the panic on overflow.
impl<const N: usize> Mul<FixedBigInt<N>> for u32 {
    type Output = FixedBigInt<N>;

    fn mul(self, rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        rhs * self
    }
}

/// In a copy of `rhs`, on the stack. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Mul<&FixedBigInt<N>> for u32 {
    type Output = FixedBigInt<N>;

    fn mul(self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        rhs.clone() * self
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

    mod words {
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

        /// Words from zero up to the largest, with a prime between.
        const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

        #[test]
        fn multiplying_by_a_word_matches_the_primitive_product() {
            for a in VALUES {
                for b in WORDS {
                    if let Some(exact) = a.checked_mul(i128::from(b)) {
                        assert_eq!(
                            FixedBigInt::<LIMBS>::from(a) * b,
                            FixedBigInt::<LIMBS>::from(exact),
                            "{a} {b}"
                        );
                    }
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to multiply with overflow")]
        fn a_product_past_the_largest_value_panics() {
            let _ = FixedBigInt::<LIMBS>::from(i128::MAX) * 2u32;
        }

        #[test]
        #[should_panic(expected = "attempt to multiply with overflow")]
        fn a_product_below_the_smallest_value_panics() {
            let _ = FixedBigInt::<LIMBS>::from(i128::MIN) * 2u32;
        }

        #[test]
        fn a_product_at_the_smallest_value_still_fits() {
            assert_eq!(
                FixedBigInt::<LIMBS>::from(i128::MIN / 2) * 2u32,
                FixedBigInt::<LIMBS>::from(i128::MIN)
            );
            assert_eq!(
                FixedBigInt::<LIMBS>::from(i128::MIN) * 1u32,
                FixedBigInt::<LIMBS>::from(i128::MIN)
            );
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (FixedBigInt::<LIMBS>::from(255i128), 0xf0f0u32);
            let expected = FixedBigInt::<LIMBS>::from(255i128 * 0xf0f0);
            assert_eq!(x.clone() * y, expected);
            assert_eq!(&x * y, expected);
            assert_eq!(y * x.clone(), expected);
            assert_eq!(y * &x, expected);
            let mut owned = x.clone();
            owned *= y;
            assert_eq!(owned, expected);
        }
    }
}
