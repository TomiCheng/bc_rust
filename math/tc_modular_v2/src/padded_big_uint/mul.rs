//! Modular multiplication of [`PaddedBigUint`].

use num_traits::Zero;
use tc_bigint_v2::{PaddedBigUint, Word};
use tc_constant_time::{Choice, ConditionallySelectable};

use super::add::add_residues;
use crate::{ModMul, NonZero};

/// `(self * rhs) mod modulus`, at the widest of the three widths, without
/// the product of twice that width: the narrower values are extended to it
/// with zeros, and the result takes it. Both operands are reduced first;
/// the product is then built from the top bit of `rhs` down, doubled at
/// each bit and given `self` where the bit is set, and reduced at every
/// step as `ModAdd` reduces. Every bit of the width takes a step, whatever
/// its value. Constant time: the widths are public.
impl ModMul for PaddedBigUint {
    type Output = Self;

    fn mod_mul(&self, rhs: &Self, modulus: &NonZero<Self>) -> Self {
        let modulus: &Self = modulus;
        let (lhs, rhs) = (self % modulus, rhs % modulus);
        // Zero at the width of `rhs`, so that the product reaches the
        // widest of the three.
        let mut product = rhs.clone();
        product.set_zero();
        for index in (0..rhs.as_limbs().len() as u32 * Word::BITS).rev() {
            product = add_residues(&product, &product, modulus);
            let sum = add_residues(&product, &lhs, modulus);
            product.conditional_assign(&sum, Choice::from_lsb(u8::from(rhs.bit(index))));
        }
        product
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{PaddedBigUint, Word};

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
            let modulus = NonZero::new(PaddedBigUint::from(*m)).unwrap();
            for a in VALUES {
                for b in VALUES {
                    let (x, y) = (PaddedBigUint::from(a), PaddedBigUint::from(b));
                    let expected = PaddedBigUint::from(expected(a, b, *m));
                    assert_eq!(x.mod_mul(&y, &modulus), expected, "{a} {b} {m}");
                }
            }
        }
    }

    #[test]
    fn a_product_wider_than_the_operands_is_reduced() {
        // 2^128 mod (2^127 - 1) is 2, and (-1)(-1) is 1 below any modulus.
        let power = PaddedBigUint::from(u64::MAX as u128 + 1);
        let modulus = NonZero::new(PaddedBigUint::from(i128::MAX as u128)).unwrap();
        assert_eq!(power.mod_mul(&power, &modulus), PaddedBigUint::from(2u8));
        let below = PaddedBigUint::from(u128::MAX - 1);
        let modulus = NonZero::new(PaddedBigUint::from(u128::MAX)).unwrap();
        assert_eq!(below.mod_mul(&below, &modulus), PaddedBigUint::from(1u8));
    }

    #[test]
    fn the_result_takes_the_widest_of_the_three_widths() {
        let wide = (u128::BITS / Word::BITS) as usize;
        let narrow = |value: u8| PaddedBigUint::from(value);
        let broad = |value: u8| PaddedBigUint::from(u128::from(value));
        for (a, b, m) in [
            (broad(7), narrow(5), narrow(9)),
            (narrow(7), broad(5), narrow(9)),
            (narrow(7), narrow(5), broad(9)),
        ] {
            let product = a.mod_mul(&b, &NonZero::new(m).unwrap());
            assert_eq!(
                (product.as_limbs().len(), product),
                (wide, PaddedBigUint::from(8u8))
            );
        }
    }
}
