//! Multiplication of [`BigInt`].

use alloc::vec::Vec;
use core::ops::{Mul, MulAssign};

use super::BigInt;
use super::sign::magnitude;
use crate::encoding::sign_fill;
use crate::limb::{conditional_negate, mul_assign_word, mul_limbs};
use crate::{Limb, Word};

/// The product of two's-complement `lhs` and `rhs`: the product of the
/// magnitudes in both lengths together, which leaves its top bit clear,
/// then the sign. Variable time.
fn product(lhs: &[Limb], rhs: &[Limb]) -> Vec<Limb> {
    let sign = sign_fill(lhs) ^ sign_fill(rhs);
    let mut product = mul_limbs(&magnitude(lhs), &magnitude(rhs));
    conditional_negate(&mut product, sign);
    product
}

/// The product in a new buffer of both lengths together, so it never
/// overflows; the result is trimmed. Schoolbook for short operands and
/// Karatsuba for long ones; a negative operand is negated into a copy
/// first. Variable time: only for public values.
impl Mul<&BigInt> for &BigInt {
    type Output = BigInt;

    fn mul(self, rhs: &BigInt) -> BigInt {
        BigInt::new(product(self.as_limbs(), rhs.as_limbs()))
    }
}

/// The product in a new buffer, as for `&self * rhs`. Variable time: only
/// for public values.
impl MulAssign<&BigInt> for BigInt {
    fn mul_assign(&mut self, rhs: &BigInt) {
        *self = &*self * rhs;
    }
}

/// The product in a new buffer, as for `&self * &rhs`. Variable time: only
/// for public values.
impl MulAssign<BigInt> for BigInt {
    fn mul_assign(&mut self, rhs: BigInt) {
        *self = &*self * &rhs;
    }
}

/// The product in a new buffer, as for `&self * rhs`. Variable time: only
/// for public values.
impl Mul<&BigInt> for BigInt {
    type Output = BigInt;

    fn mul(self, rhs: &BigInt) -> BigInt {
        &self * rhs
    }
}

/// The product in a new buffer, as for `&self * &rhs`. Variable time: only
/// for public values.
impl Mul<BigInt> for BigInt {
    type Output = BigInt;

    fn mul(self, rhs: BigInt) -> BigInt {
        &self * &rhs
    }
}

/// The product in a new buffer, as for `self * &rhs`. Variable time: only
/// for public values.
impl Mul<BigInt> for &BigInt {
    type Output = BigInt;

    fn mul(self, rhs: BigInt) -> BigInt {
        self * &rhs
    }
}

/// The magnitude multiplied by the word in one pass from the bottom, in the
/// storage of `self`, which grows by a limb only when the product needs
/// one, then the sign put back; the result is trimmed, so it cannot
/// overflow. Variable time: only for public values.
impl MulAssign<u32> for BigInt {
    fn mul_assign(&mut self, rhs: u32) {
        let mut limbs = core::mem::take(self).into_limbs();
        let sign = sign_fill(&limbs);
        conditional_negate(&mut limbs, sign);
        let carry = mul_assign_word(&mut limbs, Word::from(rhs));
        if carry != 0 {
            limbs.push(Limb::new(carry));
        }
        // a zero limb on top keeps a magnitude whose top bit is set from
        // reading as negative before the sign goes back on
        if sign_fill(&limbs) != 0 {
            limbs.push(Limb::new(0));
        }
        conditional_negate(&mut limbs, sign);
        *self = BigInt::new(limbs);
    }
}

/// In the storage of `self`, as `*=`. Variable time: only for public values.
impl Mul<u32> for BigInt {
    type Output = BigInt;

    fn mul(mut self, rhs: u32) -> BigInt {
        self *= rhs;
        self
    }
}

/// In a copy of `self`. Variable time: only for public values.
impl Mul<u32> for &BigInt {
    type Output = BigInt;

    fn mul(self, rhs: u32) -> BigInt {
        self.clone() * rhs
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Variable time:
/// only for public values.
impl Mul<BigInt> for u32 {
    type Output = BigInt;

    fn mul(self, rhs: BigInt) -> BigInt {
        rhs * self
    }
}

/// In a copy of `rhs`. Variable time: only for public values.
impl Mul<&BigInt> for u32 {
    type Output = BigInt;

    fn mul(self, rhs: &BigInt) -> BigInt {
        rhs.clone() * self
    }
}

#[cfg(test)]
mod tests {
    use crate::BigInt;

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
                        &BigInt::from(a) * &BigInt::from(b),
                        BigInt::from(product),
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
                let (x, y) = (BigInt::from(a), BigInt::from(b));
                let one_more = &y + &BigInt::from(1i8);
                assert_eq!(&x * &one_more, &(&x * &y) + &x, "{a} {b}");
            }
        }
    }

    #[test]
    fn long_products_follow_the_same_law() {
        // long enough for Karatsuba, with every sign
        let x = BigInt::from(i128::MIN) << 4000;
        for y in [BigInt::from(i128::MAX) << 3000, BigInt::from(-7i8) << 9000] {
            let one_more = &y + &BigInt::from(1i8);
            assert_eq!(&x * &one_more, &(&x * &y) + &x);
            assert_eq!(&(-&x) * &y, -(&x * &y));
        }
    }

    #[test]
    fn a_zero_operand_leaves_no_limbs() {
        assert!(
            (BigInt::default() * BigInt::from(-5i8))
                .as_limbs()
                .is_empty()
        );
        assert!(
            (BigInt::from(-5i8) * BigInt::default())
                .as_limbs()
                .is_empty()
        );
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (BigInt::from(-255i16), BigInt::from(0xf0f0i32));
        let expected = BigInt::from(-255i32 * 0xf0f0);
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

    mod words {
        use crate::BigInt;

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
        fn multiplying_by_a_word_matches_the_primitive_product_or_the_wide_one() {
            for a in VALUES {
                for b in WORDS {
                    let expected = match a.checked_mul(i128::from(b)) {
                        Some(exact) => BigInt::from(exact),
                        None => BigInt::from(a) * BigInt::from(b),
                    };
                    assert_eq!(BigInt::from(a) * b, expected, "{a} {b}");
                }
            }
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (BigInt::from(255i128), 0xf0f0u32);
            let expected = BigInt::from(255i128 * 0xf0f0);
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
