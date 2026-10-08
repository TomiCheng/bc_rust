//! Modular multiplication of [`BigUint`].

use tc_bigint_v2::BigUint;

use crate::{ModMul, NonZero};

/// `(self * rhs) mod modulus`. Both operands are reduced first, so the
/// product, which is formed in full, has at most twice the length of the
/// modulus. Variable time: only for public values; secret ones go through
/// the `ModMul` of [`PaddedBigUint`].
///
/// [`PaddedBigUint`]: tc_bigint_v2::PaddedBigUint
impl ModMul for BigUint {
    type Output = Self;

    fn mod_mul(&self, rhs: &Self, modulus: &NonZero<Self>) -> Self {
        let modulus: &Self = modulus;
        ((self % modulus) * (rhs % modulus)) % modulus
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::BigUint;

    use crate::{ModMul, NonZero};

    const VALUES: [u128; 8] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX - 1,
        u128::MAX,
    ];

    /// `(a * b) mod m`, by doubling and adding, as `u128` cannot hold the
    /// product.
    fn expected(a: u128, b: u128, m: u128) -> u128 {
        let add = |x: u128, y: u128| if x >= m - y { x - (m - y) } else { x + y };
        let (a, b) = (a % m, b % m);
        (0..u128::BITS).rev().fold(0, |product, index| {
            let doubled = add(product, product);
            if b >> index & 1 == 1 {
                add(doubled, a)
            } else {
                doubled
            }
        })
    }

    #[test]
    fn the_product_is_the_primitive_product_reduced_by_the_modulus() {
        for m in &VALUES[1..] {
            let modulus = NonZero::new(BigUint::from(*m)).unwrap();
            for a in VALUES {
                for b in VALUES {
                    let (x, y) = (BigUint::from(a), BigUint::from(b));
                    let expected = BigUint::from(expected(a, b, *m));
                    assert_eq!(x.mod_mul(&y, &modulus), expected, "{a} {b} {m}");
                }
            }
        }
    }

    #[test]
    fn a_product_wider_than_the_operands_is_reduced() {
        // 2^128 mod (2^127 - 1) is 2, and (-1)(-1) is 1 below any modulus.
        let power = BigUint::from(u64::MAX as u128 + 1);
        let modulus = NonZero::new(BigUint::from(i128::MAX as u128)).unwrap();
        assert_eq!(power.mod_mul(&power, &modulus), BigUint::from(2u8));
        let below = BigUint::from(u128::MAX - 1);
        let modulus = NonZero::new(BigUint::from(u128::MAX)).unwrap();
        assert_eq!(below.mod_mul(&below, &modulus), BigUint::from(1u8));
    }
}
