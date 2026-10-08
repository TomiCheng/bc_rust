//! Multiplication of [`PaddedBigUint`].

use core::ops::{Mul, MulAssign};

use super::PaddedBigUint;
use crate::limb::mul_assign_limbs;

/// In place at the wider width, schoolbook: the narrower operand is
/// extended to it with zeros, and the result takes it. Panics on overflow
/// in every build, unlike the primitive integers, whose check depends on
/// the profile. Constant time, apart from that panic.
impl MulAssign<&PaddedBigUint> for PaddedBigUint {
    fn mul_assign(&mut self, rhs: &PaddedBigUint) {
        self.widen(rhs.as_limbs().len());
        let lost = mul_assign_limbs(self.limbs_mut(), rhs.as_limbs(), 0);
        assert!(!lost, "attempt to multiply with overflow");
    }
}

/// The same as `*= &rhs`. Constant time, apart from the panic on overflow.
impl MulAssign<PaddedBigUint> for PaddedBigUint {
    fn mul_assign(&mut self, rhs: PaddedBigUint) {
        *self *= &rhs;
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic
/// on overflow.
impl Mul<&PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn mul(mut self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self *= rhs;
        self
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic
/// on overflow.
impl Mul<PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn mul(mut self, rhs: PaddedBigUint) -> PaddedBigUint {
        self *= &rhs;
        self
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Constant
/// time, apart from the panic on overflow.
impl Mul<PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn mul(self, mut rhs: PaddedBigUint) -> PaddedBigUint {
        rhs *= self;
        rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time, apart from the panic on overflow.
impl Mul<&PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn mul(self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self.clone_for(rhs) * rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::{PaddedBigUint, Word};

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
                        &PaddedBigUint::from(a) * &PaddedBigUint::from(b),
                        PaddedBigUint::from(product),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_narrower_operand_is_zero_extended_to_the_wider_width() {
        let product = PaddedBigUint::from(2u8) * PaddedBigUint::from(1u128 << 100);
        assert_eq!(product, PaddedBigUint::from(1u128 << 101));
        assert_eq!(product.as_limbs().len(), (u128::BITS / Word::BITS) as usize);
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            PaddedBigUint::from(255u128),
            PaddedBigUint::from(0xf0f0u128),
        );
        let expected = PaddedBigUint::from(255u128 * 0xf0f0);
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
    fn a_product_past_the_wider_width_panics() {
        let _ = PaddedBigUint::from(u64::MAX) * PaddedBigUint::from(2u8);
    }
}
