//! Multiplication of [`FixedBigUint`].

use core::ops::{Mul, MulAssign};

use super::FixedBigUint;
use crate::Word;
use crate::limb::{mul_assign_limbs, mul_assign_word};

impl<const N: usize> FixedBigUint<N> {
    /// Multiplies by `rhs` in place, keeping the low limbs of the product, and
    /// returns whether that overflowed. Constant time.
    pub(super) fn overflowing_mul_assign(&mut self, rhs: &Self) -> bool {
        mul_assign_limbs(self.limbs_mut(), rhs.as_limbs(), 0)
    }
}

/// In place, schoolbook over the `N` limbs; panics on overflow in every
/// build, unlike the primitive integers, whose check depends on the
/// profile. Constant time, apart from that panic.
impl<const N: usize> MulAssign<&FixedBigUint<N>> for FixedBigUint<N> {
    fn mul_assign(&mut self, rhs: &FixedBigUint<N>) {
        let overflowed = self.overflowing_mul_assign(rhs);
        assert!(!overflowed, "attempt to multiply with overflow");
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

/// Multiplies each limb by the word in one pass from the bottom, in place;
/// panics on overflow in every build, unlike the primitive integers, whose
/// check depends on the profile. Constant time, apart from that panic.
impl<const N: usize> MulAssign<u32> for FixedBigUint<N> {
    fn mul_assign(&mut self, rhs: u32) {
        let carry = mul_assign_word(self.limbs_mut(), Word::from(rhs));
        assert!(carry == 0, "attempt to multiply with overflow");
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Mul<u32> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn mul(mut self, rhs: u32) -> FixedBigUint<N> {
        self *= rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Mul<u32> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn mul(self, rhs: u32) -> FixedBigUint<N> {
        self.clone() * rhs
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Constant time,
/// apart from the panic on overflow.
impl<const N: usize> Mul<FixedBigUint<N>> for u32 {
    type Output = FixedBigUint<N>;

    fn mul(self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        rhs * self
    }
}

/// In a copy of `rhs`, on the stack. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Mul<&FixedBigUint<N>> for u32 {
    type Output = FixedBigUint<N>;

    fn mul(self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        rhs.clone() * self
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

    mod words {
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

        /// Words from zero up to the largest, with a prime between.
        const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

        #[test]
        fn multiplying_by_a_word_matches_the_primitive_product() {
            for a in VALUES {
                for b in WORDS {
                    if let Some(exact) = a.checked_mul(u128::from(b)) {
                        assert_eq!(
                            FixedBigUint::<LIMBS>::from(a) * b,
                            FixedBigUint::<LIMBS>::from(exact),
                            "{a} {b}"
                        );
                    }
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to multiply with overflow")]
        fn a_product_past_the_top_panics() {
            let _ = FixedBigUint::<LIMBS>::from(u128::MAX) * 2u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (FixedBigUint::<LIMBS>::from(255u128), 0xf0f0u32);
            let expected = FixedBigUint::<LIMBS>::from(255u128 * 0xf0f0);
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
