//! Constant-time comparison of [`PaddedBigUint`].

use core::cmp::Ordering;
use tc_constant_time::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeOrd};

use super::PaddedBigUint;
use crate::Limb;
use crate::limb::{ct_eq_extended, ct_lt_extended};

/// Compares values, so a narrower operand counts as zero-extended.
/// Constant time: the widths only decide how far the comparison runs.
impl ConstantTimeEq for PaddedBigUint {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        ct_eq_extended(self.as_limbs(), 0, rhs.as_limbs(), 0)
    }
}

/// Compares values at any width through [`ConstantTimeEq::ct_eq`], so `==`
/// is constant time.
impl PartialEq for PaddedBigUint {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl Eq for PaddedBigUint {}

/// Compares values, so a narrower operand counts as zero-extended.
/// Constant time: the widths only decide how far the comparison runs.
impl ConstantTimeOrd for PaddedBigUint {
    fn ct_lt(&self, rhs: &Self) -> Choice {
        ct_lt_extended(self.as_limbs(), 0, rhs.as_limbs(), 0, false)
    }
}

/// Orders values at any width through [`ConstantTimeOrd::ct_lt`] and
/// `ct_eq`, so it is constant time.
impl PartialOrd for PaddedBigUint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Orders values at any width through [`ConstantTimeOrd::ct_lt`] and
/// `ct_eq`, so it is constant time.
impl Ord for PaddedBigUint {
    fn cmp(&self, other: &Self) -> Ordering {
        let less = self.ct_lt(other).unwrap_u8() == 1;
        let equal = self.ct_eq(other).unwrap_u8() == 1;
        match (less, equal) {
            (true, _) => Ordering::Less,
            (false, true) => Ordering::Equal,
            (false, false) => Ordering::Greater,
        }
    }
}

/// Takes the wider of the two widths, the narrower operand extended with
/// zeros, so that the width of the result does not show the choice:
/// `conditional_select` gives the value of `a` or `b` at that width,
/// `conditional_assign` widens `self` to it, and `conditional_swap` widens
/// both. Constant time: the widths are public.
impl ConditionallySelectable for PaddedBigUint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut selected = a.clone_for(b);
        selected.conditional_assign(b, choice);
        selected
    }

    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        self.widen(other.as_limbs().len());
        let fill = Limb::new(0);
        for (index, limb) in self.limbs_mut().iter_mut().enumerate() {
            let source = other.as_limbs().get(index).copied().unwrap_or(fill);
            limb.conditional_assign(&source, choice);
        }
    }

    fn conditional_swap(a: &mut Self, b: &mut Self, choice: Choice) {
        let width = a.as_limbs().len().max(b.as_limbs().len());
        a.widen(width);
        b.widen(width);
        for (left, right) in a.limbs_mut().iter_mut().zip(b.limbs_mut()) {
            Limb::conditional_swap(left, right, choice);
        }
    }
}

#[cfg(test)]
mod tests {
    use tc_constant_time::{ConstantTimeEq, ConstantTimeOrd};

    use super::PaddedBigUint;

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

    fn equal<T: ConstantTimeEq>(a: &T, b: &T) -> bool {
        a.ct_eq(b).unwrap_u8() == 1
    }

    #[test]
    fn values_of_different_widths_compare_by_value() {
        assert!(equal(
            &PaddedBigUint::from(1u8),
            &PaddedBigUint::from(1u128)
        ));
        assert!(equal(
            &PaddedBigUint::from(1u128),
            &PaddedBigUint::from(1u8)
        ));
        assert!(!equal(
            &PaddedBigUint::from(1u8),
            &PaddedBigUint::from(2u128)
        ));
    }

    #[test]
    fn a_set_bit_beyond_the_narrower_width_is_a_difference() {
        assert!(!equal(
            &PaddedBigUint::from(u64::MAX),
            &PaddedBigUint::from(u128::MAX)
        ));
    }

    #[test]
    fn zero_width_equals_zero() {
        assert!(equal(&PaddedBigUint::default(), &PaddedBigUint::from(0u64)));
    }

    #[test]
    fn ordering_matches_the_primitive_one() {
        for a in VALUES {
            for b in VALUES {
                assert_eq!(
                    PaddedBigUint::from(a).cmp(&PaddedBigUint::from(b)),
                    a.cmp(&b),
                    "{a} {b}"
                );
            }
        }
    }

    #[test]
    fn the_constant_time_comparisons_agree_with_the_ordering() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (PaddedBigUint::from(a), PaddedBigUint::from(b));
                assert_eq!(x.ct_lt(&y).unwrap_u8() == 1, a < b, "{a} {b}");
                assert_eq!(x.ct_gt(&y).unwrap_u8() == 1, a > b, "{a} {b}");
                assert_eq!(x.ct_le(&y).unwrap_u8() == 1, a <= b, "{a} {b}");
                assert_eq!(x.ct_ge(&y).unwrap_u8() == 1, a >= b, "{a} {b}");
            }
        }
    }

    #[test]
    fn values_at_different_widths_are_ordered_by_value() {
        assert!(PaddedBigUint::from(2u8) > PaddedBigUint::from(1u128));
        assert!(PaddedBigUint::from(1u128) < PaddedBigUint::from(2u8));
        assert!(PaddedBigUint::from(255u8) < PaddedBigUint::from(u128::MAX));
    }

    #[test]
    fn selecting_takes_the_wider_width_whatever_the_choice() {
        use tc_constant_time::{Choice, ConditionallySelectable};

        let wide = (u128::BITS / crate::Word::BITS) as usize;
        let (a, b) = (PaddedBigUint::from(5u8), PaddedBigUint::from(7u128));
        for bit in [0, 1] {
            let choice = Choice::from_lsb(bit);
            let chosen = if bit == 1 { &b } else { &a };
            let selected = PaddedBigUint::conditional_select(&a, &b, choice);
            assert_eq!((&selected, selected.as_limbs().len()), (chosen, wide));
            let mut assigned = a.clone();
            assigned.conditional_assign(&b, choice);
            assert_eq!((&assigned, assigned.as_limbs().len()), (chosen, wide));
            let (mut left, mut right) = (a.clone(), b.clone());
            PaddedBigUint::conditional_swap(&mut left, &mut right, choice);
            assert_eq!((&left, &right), if bit == 1 { (&b, &a) } else { (&a, &b) });
            assert_eq!(
                (left.as_limbs().len(), right.as_limbs().len()),
                (wide, wide)
            );
        }
    }
}
