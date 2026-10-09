//! Addition of [`PaddedBigInt`].

use core::ops::{Add, AddAssign};

use super::PaddedBigInt;
use crate::Word;
use crate::encoding::sign_fill;
use crate::limb::{add_assign_limbs, add_assign_word, signed_add_overflowed};

impl PaddedBigInt {
    /// Adds `rhs` in place at the wider width, wrapping as two's complement,
    /// and returns whether that overflowed. Constant time.
    pub(super) fn overflowing_add_assign(&mut self, rhs: &Self) -> bool {
        self.widen(rhs.as_limbs().len());
        let (self_sign, rhs_sign) = (sign_fill(self.as_limbs()), sign_fill(rhs.as_limbs()));
        add_assign_limbs(self.limbs_mut(), rhs.as_limbs(), rhs_sign);
        let sum_sign = sign_fill(self.as_limbs());
        signed_add_overflowed(self_sign, rhs_sign, sum_sign)
    }
}

/// In place at the wider width: the narrower operand is extended to it
/// with its sign, and the result takes it. Panics on overflow in every
/// build, unlike the primitive integers, whose check depends on the
/// profile. Constant time, apart from that panic.
impl AddAssign<&PaddedBigInt> for PaddedBigInt {
    fn add_assign(&mut self, rhs: &PaddedBigInt) {
        let overflowed = self.overflowing_add_assign(rhs);
        assert!(!overflowed, "attempt to add with overflow");
    }
}

/// The same as `+= &rhs`. Constant time, apart from the panic on overflow.
impl AddAssign<PaddedBigInt> for PaddedBigInt {
    fn add_assign(&mut self, rhs: PaddedBigInt) {
        *self += &rhs;
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic
/// on overflow.
impl Add<&PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn add(mut self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self += rhs;
        self
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic
/// on overflow.
impl Add<PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn add(mut self, rhs: PaddedBigInt) -> PaddedBigInt {
        self += &rhs;
        self
    }
}

/// In the storage of `rhs`, as addition is commutative. Constant time,
/// apart from the panic on overflow.
impl Add<PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn add(self, mut rhs: PaddedBigInt) -> PaddedBigInt {
        rhs += self;
        rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time, apart from the panic on overflow.
impl Add<&PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn add(self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self.clone_for(rhs) + rhs
    }
}

/// Adds the word, a positive value whatever its top bit, to the low limb
/// and carries on up, in place at the width of `self`, which the word does
/// not widen, as two's complement; panics on overflow in every build,
/// unlike the primitive integers, whose check depends on the profile.
/// Constant time, apart from that panic.
impl AddAssign<u32> for PaddedBigInt {
    fn add_assign(&mut self, rhs: u32) {
        let sign = sign_fill(self.as_limbs());
        let carry = add_assign_word(self.limbs_mut(), Word::from(rhs));
        // the limb above, as if `self` were one limb longer; the sum fits
        // when it only repeats the sign below it
        let overflowed = sign.wrapping_add(carry) != sign_fill(self.as_limbs());
        assert!(!overflowed, "attempt to add with overflow");
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic on
/// overflow.
impl Add<u32> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn add(mut self, rhs: u32) -> PaddedBigInt {
        self += rhs;
        self
    }
}

/// In a copy of `self`. Constant time, apart from the panic on overflow.
impl Add<u32> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn add(self, rhs: u32) -> PaddedBigInt {
        self.clone() + rhs
    }
}

/// In the storage of `rhs`, as addition is commutative. Constant time, apart
/// from the panic on overflow.
impl Add<PaddedBigInt> for u32 {
    type Output = PaddedBigInt;

    fn add(self, rhs: PaddedBigInt) -> PaddedBigInt {
        rhs + self
    }
}

/// In a copy of `rhs`. Constant time, apart from the panic on overflow.
impl Add<&PaddedBigInt> for u32 {
    type Output = PaddedBigInt;

    fn add(self, rhs: &PaddedBigInt) -> PaddedBigInt {
        rhs.clone() + self
    }
}

#[cfg(test)]
mod tests {
    use crate::PaddedBigInt;
    use crate::Word;

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
    fn sums_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(sum) = a.checked_add(b) {
                    assert_eq!(
                        &PaddedBigInt::from(a) + &PaddedBigInt::from(b),
                        PaddedBigInt::from(sum),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (PaddedBigInt::from(255i128), PaddedBigInt::from(0xf0f0i128));
        let expected = PaddedBigInt::from(255i128 + 0xf0f0);
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
    fn a_narrower_operand_is_sign_extended_to_the_wider_width() {
        let sum = PaddedBigInt::from(-1i8) + PaddedBigInt::from(1i128);
        assert_eq!(sum, PaddedBigInt::from(0i8));
        assert_eq!(sum.as_limbs().len(), (i128::BITS / Word::BITS) as usize);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn passing_the_largest_value_at_the_width_panics() {
        let _ = PaddedBigInt::from(i128::MAX) + PaddedBigInt::from(1i128);
    }

    mod words {
        use crate::PaddedBigInt;

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

        /// Words from zero up to the largest, with a prime between.
        const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

        #[test]
        fn adding_a_word_matches_the_primitive_sum() {
            for a in VALUES {
                for b in WORDS {
                    if let Some(exact) = a.checked_add(i128::from(b)) {
                        assert_eq!(
                            PaddedBigInt::from(a) + b,
                            PaddedBigInt::from(exact),
                            "{a} {b}"
                        );
                    }
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to add with overflow")]
        fn a_sum_past_the_largest_value_panics() {
            let _ = PaddedBigInt::from(i128::MAX) + 1u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (PaddedBigInt::from(255i128), 0xf0f0u32);
            let expected = PaddedBigInt::from(255i128 + 0xf0f0);
            assert_eq!(x.clone() + y, expected);
            assert_eq!(&x + y, expected);
            assert_eq!(y + x.clone(), expected);
            assert_eq!(y + &x, expected);
            let mut owned = x.clone();
            owned += y;
            assert_eq!(owned, expected);
        }

        #[test]
        fn a_word_with_its_top_bit_set_still_counts_as_positive() {
            let one_limb = |word: crate::Word| {
                PaddedBigInt::new(alloc::vec![crate::Limb::new(word)].into_boxed_slice())
            };
            let smallest: crate::Word = 1 << (crate::Word::BITS - 1);
            let raised = smallest.wrapping_add(crate::Word::from(u32::MAX));
            // the smallest value of one limb, and the largest word on it
            assert_eq!(one_limb(smallest) + u32::MAX, one_limb(raised));
            assert_eq!(one_limb(raised) - u32::MAX, one_limb(smallest));
        }

        #[test]
        fn the_word_keeps_the_width_of_self() {
            let narrow = PaddedBigInt::from(5i8);
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
            let _ = <PaddedBigInt as num_traits::Zero>::zero() + 1u32;
        }
    }
}
