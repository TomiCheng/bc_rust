//! Modular subtraction of [`BigUint`].

use tc_bigint_v2::BigUint;

use crate::{ModSub, NonZero};

/// `(self - rhs) mod modulus`, never negative. Both operands are reduced
/// first; when the residue of `rhs` is the larger, the gap between them is
/// taken from the modulus instead. Variable time: only for public values;
/// secret ones go through the `ModSub` of [`PaddedBigUint`].
///
/// [`PaddedBigUint`]: tc_bigint_v2::PaddedBigUint
impl ModSub for BigUint {
    type Output = Self;

    fn mod_sub(&self, rhs: &Self, modulus: &NonZero<Self>) -> Self {
        let modulus: &Self = modulus;
        let (lhs, rhs) = (self % modulus, rhs % modulus);
        if lhs >= rhs {
            lhs - rhs
        } else {
            modulus - (rhs - lhs)
        }
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::BigUint;

    use crate::{ModSub, NonZero};

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

    /// `(a - b) mod m`, never negative.
    fn expected(a: u128, b: u128, m: u128) -> u128 {
        let (a, b) = (a % m, b % m);
        if a >= b { a - b } else { m - (b - a) }
    }

    #[test]
    fn the_difference_is_the_primitive_difference_reduced_by_the_modulus() {
        for m in &VALUES[1..] {
            let modulus = NonZero::new(BigUint::from(*m)).unwrap();
            for a in VALUES {
                for b in VALUES {
                    let (x, y) = (BigUint::from(a), BigUint::from(b));
                    let expected = BigUint::from(expected(a, b, *m));
                    assert_eq!(x.mod_sub(&y, &modulus), expected, "{a} {b} {m}");
                }
            }
        }
    }

    #[test]
    fn a_difference_below_zero_is_raised_by_the_modulus() {
        let modulus = NonZero::new(BigUint::from(u128::MAX)).unwrap();
        let (x, y) = (BigUint::from(1u8), BigUint::from(2u8));
        assert_eq!(x.mod_sub(&y, &modulus), BigUint::from(u128::MAX - 1));
    }
}
