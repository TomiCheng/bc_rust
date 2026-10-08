//! Modular addition of [`BigUint`].

use tc_bigint_v2::BigUint;

use crate::{ModAdd, NonZero};

/// `(self + rhs) mod modulus`. Both operands are reduced first, and the sum
/// of the two residues, which grows rather than overflows, has the modulus
/// taken away when it reaches it. Variable time: only for public values;
/// secret ones go through the `ModAdd` of [`PaddedBigUint`].
///
/// [`PaddedBigUint`]: tc_bigint_v2::PaddedBigUint
impl ModAdd for BigUint {
    type Output = Self;

    fn mod_add(&self, rhs: &Self, modulus: &NonZero<Self>) -> Self {
        let modulus: &Self = modulus;
        let sum = (self % modulus) + (rhs % modulus);
        if &sum >= modulus { sum - modulus } else { sum }
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::BigUint;

    use crate::{ModAdd, NonZero};

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

    /// `(a + b) mod m`, without the sum overflowing `u128`.
    fn expected(a: u128, b: u128, m: u128) -> u128 {
        let (a, b) = (a % m, b % m);
        if a >= m - b { a - (m - b) } else { a + b }
    }

    #[test]
    fn the_sum_is_the_primitive_sum_reduced_by_the_modulus() {
        for m in &VALUES[1..] {
            let modulus = NonZero::new(BigUint::from(*m)).unwrap();
            for a in VALUES {
                for b in VALUES {
                    let (x, y) = (BigUint::from(a), BigUint::from(b));
                    let expected = BigUint::from(expected(a, b, *m));
                    assert_eq!(x.mod_add(&y, &modulus), expected, "{a} {b} {m}");
                }
            }
        }
    }

    #[test]
    fn a_sum_longer_than_the_operands_is_still_reduced() {
        let modulus = NonZero::new(BigUint::from(u128::MAX)).unwrap();
        let x = BigUint::from(u128::MAX - 1);
        assert_eq!(x.mod_add(&x, &modulus), BigUint::from(u128::MAX - 2));
    }
}
