//! Multiplication of [`PaddedBigInt`].

use core::ops::{Mul, MulAssign};

use super::PaddedBigInt;
use crate::Word;
use crate::encoding::sign_fill;
use crate::limb::{conditional_negate, ct_eq_extended, mul_assign_word, signed_mul_assign_limbs};

impl PaddedBigInt {
    /// Multiplies by `rhs` in place at the wider width, keeping the low limbs
    /// of the product, and returns whether that overflowed. Constant time.
    pub(super) fn overflowing_mul_assign(&mut self, rhs: &Self) -> bool {
        self.widen(rhs.as_limbs().len());
        signed_mul_assign_limbs(self.limbs_mut(), rhs.as_limbs())
    }
}

/// In place at the wider width, schoolbook on the magnitudes, then the
/// sign: the narrower operand is extended to it with its sign, and the
/// result takes it. Panics on overflow in every build, unlike the primitive
/// integers, whose check depends on the profile. Constant time, apart from
/// that panic.
impl MulAssign<&PaddedBigInt> for PaddedBigInt {
    fn mul_assign(&mut self, rhs: &PaddedBigInt) {
        let overflowed = self.overflowing_mul_assign(rhs);
        assert!(!overflowed, "attempt to multiply with overflow");
    }
}

/// The same as `*= &rhs`. Constant time, apart from the panic on overflow.
impl MulAssign<PaddedBigInt> for PaddedBigInt {
    fn mul_assign(&mut self, rhs: PaddedBigInt) {
        *self *= &rhs;
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic
/// on overflow.
impl Mul<&PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn mul(mut self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self *= rhs;
        self
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic
/// on overflow.
impl Mul<PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn mul(mut self, rhs: PaddedBigInt) -> PaddedBigInt {
        self *= &rhs;
        self
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Constant
/// time, apart from the panic on overflow.
impl Mul<PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn mul(self, mut rhs: PaddedBigInt) -> PaddedBigInt {
        rhs *= self;
        rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time, apart from the panic on overflow.
impl Mul<&PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn mul(self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self.clone_for(rhs) * rhs
    }
}

/// The magnitude multiplied by the word in one pass from the bottom, then
/// the sign put back, in place at the width of `self`, which the word does
/// not widen; panics on overflow in every build, unlike the primitive
/// integers, whose check depends on the profile. Constant time, apart from
/// that panic.
impl MulAssign<u32> for PaddedBigInt {
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
impl Mul<u32> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn mul(mut self, rhs: u32) -> PaddedBigInt {
        self *= rhs;
        self
    }
}

/// In a copy of `self`. Constant time, apart from the panic on overflow.
impl Mul<u32> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn mul(self, rhs: u32) -> PaddedBigInt {
        self.clone() * rhs
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Constant time,
/// apart from the panic on overflow.
impl Mul<PaddedBigInt> for u32 {
    type Output = PaddedBigInt;

    fn mul(self, rhs: PaddedBigInt) -> PaddedBigInt {
        rhs * self
    }
}

/// In a copy of `rhs`. Constant time, apart from the panic on overflow.
impl Mul<&PaddedBigInt> for u32 {
    type Output = PaddedBigInt;

    fn mul(self, rhs: &PaddedBigInt) -> PaddedBigInt {
        rhs.clone() * self
    }
}

#[cfg(test)]
mod tests {
    use crate::{PaddedBigInt, Word};

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
                        &PaddedBigInt::from(a) * &PaddedBigInt::from(b),
                        PaddedBigInt::from(product),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_narrower_operand_is_sign_extended_to_the_wider_width() {
        let product = PaddedBigInt::from(-2i8) * PaddedBigInt::from(1i128 << 100);
        assert_eq!(product, PaddedBigInt::from(-(1i128 << 101)));
        assert_eq!(product.as_limbs().len(), (i128::BITS / Word::BITS) as usize);
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (PaddedBigInt::from(-255i128), PaddedBigInt::from(0xf0f0i128));
        let expected = PaddedBigInt::from(-255i128 * 0xf0f0);
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
        let _ = PaddedBigInt::from(i128::MIN) * PaddedBigInt::from(-1i8);
    }

    mod words {
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

        /// Words from zero up to the largest, with a prime between.
        const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

        #[test]
        fn multiplying_by_a_word_matches_the_primitive_product() {
            for a in VALUES {
                for b in WORDS {
                    if let Some(exact) = a.checked_mul(i128::from(b)) {
                        assert_eq!(
                            PaddedBigInt::from(a) * b,
                            PaddedBigInt::from(exact),
                            "{a} {b}"
                        );
                    }
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to multiply with overflow")]
        fn a_product_past_the_largest_value_panics() {
            let _ = PaddedBigInt::from(i128::MAX) * 2u32;
        }

        #[test]
        #[should_panic(expected = "attempt to multiply with overflow")]
        fn a_product_below_the_smallest_value_panics() {
            let _ = PaddedBigInt::from(i128::MIN) * 2u32;
        }

        #[test]
        fn a_product_at_the_smallest_value_still_fits() {
            assert_eq!(
                PaddedBigInt::from(i128::MIN / 2) * 2u32,
                PaddedBigInt::from(i128::MIN)
            );
            assert_eq!(
                PaddedBigInt::from(i128::MIN) * 1u32,
                PaddedBigInt::from(i128::MIN)
            );
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (PaddedBigInt::from(255i128), 0xf0f0u32);
            let expected = PaddedBigInt::from(255i128 * 0xf0f0);
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
