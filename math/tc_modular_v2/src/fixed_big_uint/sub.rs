//! Modular subtraction of [`FixedBigUint`].

use num_traits::WrappingAdd;
use num_traits::ops::overflowing::OverflowingSub;
use tc_bigint_v2::FixedBigUint;
use tc_constant_time::{Choice, ConditionallySelectable};

use crate::{ModSub, NonZero};

/// `(self - rhs) mod modulus`, never negative. Both operands are reduced
/// first; when the difference of the two residues borrows, it has wrapped
/// around the `N` limbs, and adding the modulus wraps it back into range.
/// Constant time.
impl<const N: usize> ModSub for FixedBigUint<N> {
    type Output = Self;

    fn mod_sub(&self, rhs: &Self, modulus: &NonZero<Self>) -> Self {
        let modulus: &Self = modulus;
        let (difference, borrowed) = (self % modulus).overflowing_sub(&(rhs % modulus));
        let raised = difference.wrapping_add(modulus);
        Self::conditional_select(&difference, &raised, Choice::from_lsb(u8::from(borrowed)))
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{FixedBigUint, Word};

    use crate::{ModSub, NonZero};

    /// The limbs of 128 bits, to compare against `u128`.
    const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

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
            let modulus = NonZero::new(FixedBigUint::<LIMBS>::from(*m)).unwrap();
            for a in VALUES {
                for b in VALUES {
                    let (x, y) = (
                        FixedBigUint::<LIMBS>::from(a),
                        FixedBigUint::<LIMBS>::from(b),
                    );
                    let expected = FixedBigUint::<LIMBS>::from(expected(a, b, *m));
                    assert_eq!(x.mod_sub(&y, &modulus), expected, "{a} {b} {m}");
                }
            }
        }
    }

    #[test]
    fn a_difference_below_zero_is_raised_by_the_modulus() {
        let modulus = NonZero::new(FixedBigUint::<LIMBS>::from(u128::MAX)).unwrap();
        let (x, y) = (
            FixedBigUint::<LIMBS>::from(1u8),
            FixedBigUint::<LIMBS>::from(2u8),
        );
        assert_eq!(
            x.mod_sub(&y, &modulus),
            FixedBigUint::<LIMBS>::from(u128::MAX - 1)
        );
    }
}
