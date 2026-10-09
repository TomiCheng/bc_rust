//! Multiplication of [`BigUint`].

use core::ops::{Mul, MulAssign};

use super::BigUint;
use crate::limb::{mul_assign_word, mul_limbs};
use crate::{Limb, Word};

/// The product in a new buffer of both lengths together, so it never
/// overflows, and trimmed; schoolbook for short operands and Karatsuba for
/// long ones. Variable time: only for public values.
impl Mul<&BigUint> for &BigUint {
    type Output = BigUint;

    fn mul(self, rhs: &BigUint) -> BigUint {
        BigUint::new(mul_limbs(self.as_limbs(), rhs.as_limbs()))
    }
}

/// The product in a new buffer, as for `&self * rhs`. Variable time: only
/// for public values.
impl MulAssign<&BigUint> for BigUint {
    fn mul_assign(&mut self, rhs: &BigUint) {
        *self = &*self * rhs;
    }
}

/// The product in a new buffer, as for `&self * &rhs`. Variable time: only
/// for public values.
impl MulAssign<BigUint> for BigUint {
    fn mul_assign(&mut self, rhs: BigUint) {
        *self = &*self * &rhs;
    }
}

/// The product in a new buffer, as for `&self * rhs`. Variable time: only
/// for public values.
impl Mul<&BigUint> for BigUint {
    type Output = BigUint;

    fn mul(self, rhs: &BigUint) -> BigUint {
        &self * rhs
    }
}

/// The product in a new buffer, as for `&self * &rhs`. Variable time: only
/// for public values.
impl Mul<BigUint> for BigUint {
    type Output = BigUint;

    fn mul(self, rhs: BigUint) -> BigUint {
        &self * &rhs
    }
}

/// The product in a new buffer, as for `self * &rhs`. Variable time: only
/// for public values.
impl Mul<BigUint> for &BigUint {
    type Output = BigUint;

    fn mul(self, rhs: BigUint) -> BigUint {
        self * &rhs
    }
}

/// Multiplies each limb by the word in one pass from the bottom, in the
/// storage of `self`, which grows by a limb only when the carry out needs
/// it, so it never overflows; the result is trimmed. Variable time: only
/// for public values.
impl MulAssign<u32> for BigUint {
    fn mul_assign(&mut self, rhs: u32) {
        let mut limbs = core::mem::take(self).into_limbs();
        let carry = mul_assign_word(&mut limbs, Word::from(rhs));
        if carry != 0 {
            limbs.push(Limb::new(carry));
        }
        *self = BigUint::new(limbs);
    }
}

/// In the storage of `self`, as `*=`. Variable time: only for public
/// values.
impl Mul<u32> for BigUint {
    type Output = BigUint;

    fn mul(mut self, rhs: u32) -> BigUint {
        self *= rhs;
        self
    }
}

/// In a copy of `self`. Variable time: only for public values.
impl Mul<u32> for &BigUint {
    type Output = BigUint;

    fn mul(self, rhs: u32) -> BigUint {
        self.clone() * rhs
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Variable
/// time: only for public values.
impl Mul<BigUint> for u32 {
    type Output = BigUint;

    fn mul(self, rhs: BigUint) -> BigUint {
        rhs * self
    }
}

/// In a copy of `rhs`. Variable time: only for public values.
impl Mul<&BigUint> for u32 {
    type Output = BigUint;

    fn mul(self, rhs: &BigUint) -> BigUint {
        rhs.clone() * self
    }
}

#[cfg(test)]
mod tests {
    use crate::BigUint;

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
                        &BigUint::from(a) * &BigUint::from(b),
                        BigUint::from(product),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn multiplying_by_one_more_adds_the_value_once_more() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigUint::from(a), BigUint::from(b));
                let one_more = &y + &BigUint::from(1u8);
                assert_eq!(&x * &one_more, &(&x * &y) + &x, "{a} {b}");
            }
        }
    }

    #[test]
    fn long_products_follow_the_same_law() {
        // long enough for Karatsuba, evenly and unevenly
        let x = BigUint::from(u128::MAX) << 4000;
        for y in [
            BigUint::from(u128::MAX) << 3000,
            BigUint::from(u128::MAX) << 9000,
        ] {
            let one_more = &y + &BigUint::from(1u8);
            assert_eq!(&x * &one_more, &(&x * &y) + &x);
        }
    }

    #[test]
    fn a_zero_operand_leaves_no_limbs() {
        assert!(
            (BigUint::default() * BigUint::from(5u8))
                .as_limbs()
                .is_empty()
        );
        assert!(
            (BigUint::from(5u8) * BigUint::default())
                .as_limbs()
                .is_empty()
        );
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (BigUint::from(255u8), BigUint::from(0xf0f0u16));
        let expected = BigUint::from(255u32 * 0xf0f0);
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

    /// Words from zero up to the largest, with a prime between.
    const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

    #[test]
    fn multiplying_by_a_word_matches_the_primitive_product() {
        for a in VALUES {
            for b in WORDS {
                if let Some(product) = a.checked_mul(u128::from(b)) {
                    assert_eq!(BigUint::from(a) * b, BigUint::from(product), "{a} {b}");
                }
            }
        }
    }

    #[test]
    fn a_product_past_the_top_grows_a_limb() {
        for b in WORDS {
            let top = BigUint::from(u128::MAX);
            assert_eq!(&top * b, &top * &BigUint::from(b), "{b}");
        }
    }

    #[test]
    fn every_form_with_a_word_gives_the_same_result() {
        let (x, y) = (BigUint::from(255u128), 0xf0f0u32);
        let expected = BigUint::from(255u128 * 0xf0f0);
        assert_eq!(x.clone() * y, expected);
        assert_eq!(&x * y, expected);
        assert_eq!(y * x.clone(), expected);
        assert_eq!(y * &x, expected);
        let mut owned = x.clone();
        owned *= y;
        assert_eq!(owned, expected);
    }
}
