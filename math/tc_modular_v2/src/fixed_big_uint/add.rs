//! Modular addition of [`FixedBigUint`].

use num_traits::ops::overflowing::{OverflowingAdd, OverflowingSub};
use tc_bigint_v2::FixedBigUint;
use tc_constant_time::{Choice, ConditionallySelectable};

use crate::{ModAdd, NonZero};
use tc_zeroize::Zeroizing;

/// `(self + rhs) mod modulus`. Both operands are reduced first; the sum of
/// the two residues may carry out of the `N` limbs, and the one subtraction
/// of the modulus that follows takes that carry into account. Constant time.
impl<const N: usize> ModAdd for FixedBigUint<N> {
    type Output = Self;

    fn mod_add(&self, rhs: &Self, modulus: &NonZero<Self>) -> Self {
        let modulus: &Self = modulus;
        // The residues are wiped once the sum is out.
        let residues = (
            Zeroizing::new(self % modulus),
            Zeroizing::new(rhs % modulus),
        );
        add_residues(&residues.0, &residues.1, modulus)
    }
}

/// `(lhs + rhs) mod modulus`, for operands already below `modulus`.
/// Constant time.
pub(super) fn add_residues<const N: usize>(
    lhs: &FixedBigUint<N>,
    rhs: &FixedBigUint<N>,
    modulus: &FixedBigUint<N>,
) -> FixedBigUint<N> {
    let (sum, carried) = lhs.overflowing_add(rhs);
    let (reduced, borrowed) = sum.overflowing_sub(modulus);
    // Both are wiped once one of them is chosen.
    let (sum, reduced) = (Zeroizing::new(sum), Zeroizing::new(reduced));
    // The sum is at least the modulus when it carried out of the limbs, or
    // when taking the modulus away did not borrow.
    let at_least_modulus =
        Choice::from_lsb(u8::from(carried)) | !Choice::from_lsb(u8::from(borrowed));
    FixedBigUint::conditional_select(&sum, &reduced, at_least_modulus)
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{FixedBigUint, Word};

    use crate::{ModAdd, NonZero};

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

    /// `(a + b) mod m`, without the sum overflowing `u128`.
    fn expected(a: u128, b: u128, m: u128) -> u128 {
        let (a, b) = (a % m, b % m);
        if a >= m - b { a - (m - b) } else { a + b }
    }

    #[test]
    fn the_sum_is_the_primitive_sum_reduced_by_the_modulus() {
        for m in &VALUES[1..] {
            let modulus = NonZero::new(FixedBigUint::<LIMBS>::from(*m)).unwrap();
            for a in VALUES {
                for b in VALUES {
                    let (x, y) = (
                        FixedBigUint::<LIMBS>::from(a),
                        FixedBigUint::<LIMBS>::from(b),
                    );
                    let expected = FixedBigUint::<LIMBS>::from(expected(a, b, *m));
                    assert_eq!(x.mod_add(&y, &modulus), expected, "{a} {b} {m}");
                }
            }
        }
    }

    #[test]
    fn a_sum_that_carries_out_of_the_limbs_is_still_reduced() {
        let modulus = NonZero::new(FixedBigUint::<LIMBS>::from(u128::MAX)).unwrap();
        let x = FixedBigUint::<LIMBS>::from(u128::MAX - 1);
        let expected = FixedBigUint::<LIMBS>::from(u128::MAX - 2);
        assert_eq!(x.mod_add(&x, &modulus), expected);
    }
}
