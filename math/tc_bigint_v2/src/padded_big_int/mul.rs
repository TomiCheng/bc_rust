//! Multiplication of [`PaddedBigInt`].

use core::ops::{Mul, MulAssign};

use super::PaddedBigInt;
use crate::limb::signed_mul_assign_limbs;

/// In place at the wider width, schoolbook on the magnitudes, then the
/// sign: the narrower operand is extended to it with its sign, and the
/// result takes it. Panics on overflow in every build, unlike the primitive
/// integers, whose check depends on the profile. Constant time, apart from
/// that panic.
impl MulAssign<&PaddedBigInt> for PaddedBigInt {
    fn mul_assign(&mut self, rhs: &PaddedBigInt) {
        self.widen(rhs.as_limbs().len());
        let overflowed = signed_mul_assign_limbs(self.limbs_mut(), rhs.as_limbs());
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
}
