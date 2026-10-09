//! Modular addition of [`PaddedBigUint`].

use num_traits::ops::overflowing::{OverflowingAdd, OverflowingSub};
use tc_bigint_v2::PaddedBigUint;
use tc_constant_time::{Choice, ConditionallySelectable};

use crate::{ModAdd, NonZero};
use tc_zeroize::Zeroizing;

/// `(self + rhs) mod modulus`, at the widest of the three widths: the
/// narrower values are extended to it with zeros, and the result takes it.
/// Both operands are reduced first; the sum of the two residues may carry
/// out of that width, and the one subtraction of the modulus that follows
/// takes that carry into account. Constant time: the widths are public.
impl ModAdd for PaddedBigUint {
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

/// `(lhs + rhs) mod modulus`, for operands already below `modulus`, at the
/// widest of the three widths. Constant time: the widths are public.
pub(super) fn add_residues(
    lhs: &PaddedBigUint,
    rhs: &PaddedBigUint,
    modulus: &PaddedBigUint,
) -> PaddedBigUint {
    let (sum, carried) = lhs.overflowing_add(rhs);
    let (reduced, borrowed) = sum.overflowing_sub(modulus);
    // Both are wiped once one of them is chosen.
    let (sum, reduced) = (Zeroizing::new(sum), Zeroizing::new(reduced));
    // The sum is at least the modulus when it carried out of the width, or
    // when taking the modulus away did not borrow.
    let at_least_modulus =
        Choice::from_lsb(u8::from(carried)) | !Choice::from_lsb(u8::from(borrowed));
    PaddedBigUint::conditional_select(&sum, &reduced, at_least_modulus)
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{PaddedBigUint, Word};

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
            let modulus = NonZero::new(PaddedBigUint::from(*m)).unwrap();
            for a in VALUES {
                for b in VALUES {
                    let (x, y) = (PaddedBigUint::from(a), PaddedBigUint::from(b));
                    let expected = PaddedBigUint::from(expected(a, b, *m));
                    assert_eq!(x.mod_add(&y, &modulus), expected, "{a} {b} {m}");
                }
            }
        }
    }

    #[test]
    fn a_sum_that_carries_out_of_the_width_is_still_reduced() {
        let modulus = NonZero::new(PaddedBigUint::from(u128::MAX)).unwrap();
        let x = PaddedBigUint::from(u128::MAX - 1);
        assert_eq!(x.mod_add(&x, &modulus), PaddedBigUint::from(u128::MAX - 2));
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
            let sum = a.mod_add(&b, &NonZero::new(m).unwrap());
            assert_eq!(
                (sum.as_limbs().len(), sum),
                (wide, PaddedBigUint::from(3u8))
            );
        }
    }
}
