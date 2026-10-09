//! Multiplication of [`PaddedBigUint`].

use core::ops::{Mul, MulAssign};

use super::PaddedBigUint;
use crate::Word;
use crate::limb::{mul_assign_limbs, mul_assign_word};

impl PaddedBigUint {
    /// Multiplies by `rhs` in place at the wider width, keeping the low limbs
    /// of the product, and returns whether that overflowed. Constant time.
    pub(super) fn overflowing_mul_assign(&mut self, rhs: &Self) -> bool {
        self.widen(rhs.as_limbs().len());
        mul_assign_limbs(self.limbs_mut(), rhs.as_limbs(), 0)
    }
}

/// In place at the wider width, schoolbook: the narrower operand is
/// extended to it with zeros, and the result takes it. Panics on overflow
/// in every build, unlike the primitive integers, whose check depends on
/// the profile. Constant time, apart from that panic.
impl MulAssign<&PaddedBigUint> for PaddedBigUint {
    fn mul_assign(&mut self, rhs: &PaddedBigUint) {
        let overflowed = self.overflowing_mul_assign(rhs);
        assert!(!overflowed, "attempt to multiply with overflow");
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

/// Multiplies each limb by the word in one pass from the bottom, in place
/// at the width of `self`, which the word does not widen; panics on
/// overflow in every build, unlike the primitive integers, whose check
/// depends on the profile. Constant time, apart from that panic.
impl MulAssign<u32> for PaddedBigUint {
    fn mul_assign(&mut self, rhs: u32) {
        let carry = mul_assign_word(self.limbs_mut(), Word::from(rhs));
        assert!(carry == 0, "attempt to multiply with overflow");
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic on
/// overflow.
impl Mul<u32> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn mul(mut self, rhs: u32) -> PaddedBigUint {
        self *= rhs;
        self
    }
}

/// In a copy of `self`. Constant time, apart from the panic on overflow.
impl Mul<u32> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn mul(self, rhs: u32) -> PaddedBigUint {
        self.clone() * rhs
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Constant time,
/// apart from the panic on overflow.
impl Mul<PaddedBigUint> for u32 {
    type Output = PaddedBigUint;

    fn mul(self, rhs: PaddedBigUint) -> PaddedBigUint {
        rhs * self
    }
}

/// In a copy of `rhs`. Constant time, apart from the panic on overflow.
impl Mul<&PaddedBigUint> for u32 {
    type Output = PaddedBigUint;

    fn mul(self, rhs: &PaddedBigUint) -> PaddedBigUint {
        rhs.clone() * self
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

    mod words {
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

        /// Words from zero up to the largest, with a prime between.
        const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

        #[test]
        fn multiplying_by_a_word_matches_the_primitive_product() {
            for a in VALUES {
                for b in WORDS {
                    if let Some(exact) = a.checked_mul(u128::from(b)) {
                        assert_eq!(
                            PaddedBigUint::from(a) * b,
                            PaddedBigUint::from(exact),
                            "{a} {b}"
                        );
                    }
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to multiply with overflow")]
        fn a_product_past_the_top_panics() {
            let _ = PaddedBigUint::from(u128::MAX) * 2u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (PaddedBigUint::from(255u128), 0xf0f0u32);
            let expected = PaddedBigUint::from(255u128 * 0xf0f0);
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
