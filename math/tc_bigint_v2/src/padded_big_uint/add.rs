//! Addition of [`PaddedBigUint`].

use core::ops::{Add, AddAssign};

use super::PaddedBigUint;
use crate::Word;
use crate::limb::{add_assign_limbs, add_assign_word};

impl PaddedBigUint {
    /// Adds `rhs` in place at the wider width, wrapping around, and returns
    /// whether that overflowed. Constant time.
    pub(super) fn overflowing_add_assign(&mut self, rhs: &Self) -> bool {
        self.widen(rhs.as_limbs().len());
        let carry = add_assign_limbs(self.limbs_mut(), rhs.as_limbs(), 0);
        carry != 0
    }
}

/// In place at the wider width: the narrower operand is extended to it
/// with zeros, and the result takes it. Panics on overflow in every
/// build, unlike the primitive integers, whose check depends on the
/// profile. Constant time, apart from that panic.
impl AddAssign<&PaddedBigUint> for PaddedBigUint {
    fn add_assign(&mut self, rhs: &PaddedBigUint) {
        let overflowed = self.overflowing_add_assign(rhs);
        assert!(!overflowed, "attempt to add with overflow");
    }
}

/// The same as `+= &rhs`. Constant time, apart from the panic on overflow.
impl AddAssign<PaddedBigUint> for PaddedBigUint {
    fn add_assign(&mut self, rhs: PaddedBigUint) {
        *self += &rhs;
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic
/// on overflow.
impl Add<&PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn add(mut self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self += rhs;
        self
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic
/// on overflow.
impl Add<PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn add(mut self, rhs: PaddedBigUint) -> PaddedBigUint {
        self += &rhs;
        self
    }
}

/// In the storage of `rhs`, as addition is commutative. Constant time,
/// apart from the panic on overflow.
impl Add<PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn add(self, mut rhs: PaddedBigUint) -> PaddedBigUint {
        rhs += self;
        rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time, apart from the panic on overflow.
impl Add<&PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn add(self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self.clone_for(rhs) + rhs
    }
}

/// Adds the word to the low limb and carries on up, in place at the width
/// of `self`, which the word does not widen; panics on overflow in every
/// build, unlike the primitive integers, whose check depends on the
/// profile. Constant time, apart from that panic.
impl AddAssign<u32> for PaddedBigUint {
    fn add_assign(&mut self, rhs: u32) {
        let carry = add_assign_word(self.limbs_mut(), Word::from(rhs));
        assert!(carry == 0, "attempt to add with overflow");
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic on
/// overflow.
impl Add<u32> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn add(mut self, rhs: u32) -> PaddedBigUint {
        self += rhs;
        self
    }
}

/// In a copy of `self`. Constant time, apart from the panic on overflow.
impl Add<u32> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn add(self, rhs: u32) -> PaddedBigUint {
        self.clone() + rhs
    }
}

/// In the storage of `rhs`, as addition is commutative. Constant time, apart
/// from the panic on overflow.
impl Add<PaddedBigUint> for u32 {
    type Output = PaddedBigUint;

    fn add(self, rhs: PaddedBigUint) -> PaddedBigUint {
        rhs + self
    }
}

/// In a copy of `rhs`. Constant time, apart from the panic on overflow.
impl Add<&PaddedBigUint> for u32 {
    type Output = PaddedBigUint;

    fn add(self, rhs: &PaddedBigUint) -> PaddedBigUint {
        rhs.clone() + self
    }
}

#[cfg(test)]
mod tests {
    use crate::PaddedBigUint;
    use crate::Word;

    const VALUES: [u128; 7] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX,
    ];

    #[test]
    fn sums_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(sum) = a.checked_add(b) {
                    assert_eq!(
                        &PaddedBigUint::from(a) + &PaddedBigUint::from(b),
                        PaddedBigUint::from(sum),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            PaddedBigUint::from(255u128),
            PaddedBigUint::from(0xf0f0u128),
        );
        let expected = PaddedBigUint::from(255u128 + 0xf0f0);
        assert_eq!(x.clone() + y.clone(), expected);
        assert_eq!(x.clone() + &y, expected);
        assert_eq!(&x + y.clone(), expected);
        assert_eq!(&x + &y, expected);
        let mut owned = x.clone();
        owned += y.clone();
        assert_eq!(owned, expected);
        let mut borrowed = x;
        borrowed += &y;
        assert_eq!(borrowed, expected);
    }

    #[test]
    fn a_narrower_operand_is_extended_to_the_wider_width() {
        let sum = PaddedBigUint::from(1u8) + PaddedBigUint::from(u64::MAX as u128);
        assert_eq!(sum, PaddedBigUint::from(1u128 << 64));
        assert_eq!(sum.as_limbs().len(), (u128::BITS / Word::BITS) as usize);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn a_carry_out_of_the_wider_width_panics() {
        let _ = PaddedBigUint::from(u64::MAX) + PaddedBigUint::from(1u8);
    }

    mod words {
        use crate::PaddedBigUint;

        const VALUES: [u128; 7] = [
            0,
            1,
            255,
            u64::MAX as u128,
            u64::MAX as u128 + 1,
            i128::MAX as u128,
            u128::MAX,
        ];

        /// Words from zero up to the largest, with a prime between.
        const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

        #[test]
        fn adding_a_word_matches_the_primitive_sum() {
            for a in VALUES {
                for b in WORDS {
                    if let Some(exact) = a.checked_add(u128::from(b)) {
                        assert_eq!(
                            PaddedBigUint::from(a) + b,
                            PaddedBigUint::from(exact),
                            "{a} {b}"
                        );
                    }
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to add with overflow")]
        fn a_sum_past_the_top_panics() {
            let _ = PaddedBigUint::from(u128::MAX) + 1u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (PaddedBigUint::from(255u128), 0xf0f0u32);
            let expected = PaddedBigUint::from(255u128 + 0xf0f0);
            assert_eq!(x.clone() + y, expected);
            assert_eq!(&x + y, expected);
            assert_eq!(y + x.clone(), expected);
            assert_eq!(y + &x, expected);
            let mut owned = x.clone();
            owned += y;
            assert_eq!(owned, expected);
        }

        #[test]
        fn the_word_keeps_the_width_of_self() {
            let narrow = PaddedBigUint::from(5u8);
            assert_eq!(
                (narrow.clone() + 7u32).as_limbs().len(),
                narrow.as_limbs().len()
            );
            assert_eq!(
                (narrow.clone() * 7u32).as_limbs().len(),
                narrow.as_limbs().len()
            );
            assert_eq!(
                (narrow.clone() / 7u32).as_limbs().len(),
                narrow.as_limbs().len()
            );
        }

        #[test]
        #[should_panic(expected = "attempt to add with overflow")]
        fn a_word_added_to_no_limbs_overflows() {
            let _ = <PaddedBigUint as num_traits::Zero>::zero() + 1u32;
        }
    }
}
