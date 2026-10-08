//! Multiplication of [`BigInt`].

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::ops::{Mul, MulAssign};

use super::BigInt;
use crate::Limb;
use crate::encoding::sign_fill;
use crate::limb::{conditional_negate, mul_limbs};

/// The magnitude of two's-complement `limbs`, in as many limbs: the limbs
/// themselves when not negative, a negated copy otherwise.
fn magnitude(limbs: &[Limb]) -> Cow<'_, [Limb]> {
    let sign = sign_fill(limbs);
    if sign == 0 {
        return Cow::Borrowed(limbs);
    }
    let mut negated = limbs.to_vec();
    conditional_negate(&mut negated, sign);
    Cow::Owned(negated)
}

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
}
