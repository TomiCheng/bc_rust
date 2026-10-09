//! Division and remainder of [`PaddedBigInt`].

use alloc::vec;
use core::ops::{Div, DivAssign, Rem, RemAssign};

use num_traits::Zero;

use super::PaddedBigInt;
use crate::encoding::sign_fill;
use crate::limb::{conditional_negate, div_assign_word, signed_div_rem_limbs};
use crate::{Limb, Word};

impl PaddedBigInt {
    /// The quotient and the remainder by `rhs` together, for the work
    /// of one division, at the wider width, the quotient truncated
    /// toward zero. Panics when `rhs` is zero, and when the quotient
    /// does not fit, which only the most negative value divided by -1
    /// does, in every build. Constant time, apart from those panics.
    pub fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        let mut quotient = self.clone_for(rhs);
        let (remainder, overflowed) = quotient.divide(rhs);
        assert!(!overflowed, "attempt to divide with overflow");
        (quotient, remainder)
    }

    /// Replaces `self` with its quotient by `rhs`, which is not zero,
    /// at the wider width, truncated toward zero, and returns the
    /// remainder at that width, which takes the sign of `self` and a
    /// new buffer holds, and whether the quotient overflowed. Constant
    /// time.
    pub(super) fn divide(&mut self, rhs: &Self) -> (Self, bool) {
        self.widen(rhs.as_limbs().len());
        let mut remainder = Self::new(vec![Limb::new(0); self.as_limbs().len()].into_boxed_slice());
        let overflowed =
            signed_div_rem_limbs(self.limbs_mut(), rhs.as_limbs(), remainder.limbs_mut());
        (remainder, overflowed)
    }
}

/// The quotient in place at the wider width, truncated toward zero: the
/// narrower operand is extended to it with its sign, and the result takes
/// it. The remainder is worked out in a new buffer, so it allocates once.
/// Panics when `rhs` is zero, and when the quotient does not fit, which
/// only the most negative value divided by -1 does, in every build.
/// Constant time, apart from those panics.
impl DivAssign<&PaddedBigInt> for PaddedBigInt {
    fn div_assign(&mut self, rhs: &PaddedBigInt) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        let (_, overflowed) = self.divide(rhs);
        assert!(!overflowed, "attempt to divide with overflow");
    }
}

/// The remainder at the wider width, with the sign of `self`, in the new
/// buffer it is worked out in. Panics when `rhs` is zero, in every build;
/// the most negative value by -1 gives zero, where the primitive integers
/// panic, as the remainder itself always fits. Constant time, apart from
/// that panic.
impl RemAssign<&PaddedBigInt> for PaddedBigInt {
    fn rem_assign(&mut self, rhs: &PaddedBigInt) {
        assert!(
            !rhs.is_zero(),
            "attempt to calculate the remainder with a divisor of zero"
        );
        *self = self.divide(rhs).0;
    }
}

/// The same as `/= &rhs`. Constant time, apart from the panics.
impl DivAssign<PaddedBigInt> for PaddedBigInt {
    fn div_assign(&mut self, rhs: PaddedBigInt) {
        *self /= &rhs;
    }
}

/// The quotient in the storage of `self`, as `/=`. Constant time, apart
/// from the panics.
impl Div<&PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn div(mut self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self /= rhs;
        self
    }
}

/// The quotient in the storage of `self`, as `/=`. Constant time, apart
/// from the panics.
impl Div<PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn div(mut self, rhs: PaddedBigInt) -> PaddedBigInt {
        self /= &rhs;
        self
    }
}

/// The quotient in a copy of `self` at the wider width. Constant time,
/// apart from the panics.
impl Div<PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn div(self, rhs: PaddedBigInt) -> PaddedBigInt {
        self.clone_for(&rhs) / &rhs
    }
}

/// The quotient in a copy of `self` at the wider width. Constant time,
/// apart from the panics.
impl Div<&PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn div(self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self.clone_for(rhs) / rhs
    }
}

/// The same as `%= &rhs`. Constant time, apart from the panics.
impl RemAssign<PaddedBigInt> for PaddedBigInt {
    fn rem_assign(&mut self, rhs: PaddedBigInt) {
        *self %= &rhs;
    }
}

/// The remainder in the storage of `self`, as `%=`. Constant time, apart
/// from the panics.
impl Rem<&PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn rem(mut self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self %= rhs;
        self
    }
}

/// The remainder in the storage of `self`, as `%=`. Constant time, apart
/// from the panics.
impl Rem<PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn rem(mut self, rhs: PaddedBigInt) -> PaddedBigInt {
        self %= &rhs;
        self
    }
}

/// The remainder in a copy of `self` at the wider width. Constant time,
/// apart from the panics.
impl Rem<PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn rem(self, rhs: PaddedBigInt) -> PaddedBigInt {
        self.clone_for(&rhs) % &rhs
    }
}

/// The remainder in a copy of `self` at the wider width. Constant time,
/// apart from the panics.
impl Rem<&PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn rem(self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self.clone_for(rhs) % rhs
    }
}

/// The quotient, truncated toward zero, in place at the width of `self`,
/// which the word does not widen: the magnitude is divided by long division
/// a bit at a time, so that no hardware division sees it, then the sign is
/// put back; it cannot overflow, as the word is positive. Panics when `rhs`
/// is zero, in every build. Constant time, apart from that panic.
impl DivAssign<u32> for PaddedBigInt {
    fn div_assign(&mut self, rhs: u32) {
        assert!(rhs != 0, "attempt to divide by zero");
        let limbs = self.limbs_mut();
        let sign = sign_fill(limbs);
        conditional_negate(limbs, sign);
        div_assign_word(limbs, Word::from(rhs));
        conditional_negate(limbs, sign);
    }
}

/// The remainder, with the sign of `self`, in place at the width of `self`,
/// which the word does not widen, by the same long division of the
/// magnitude a bit at a time. Panics when `rhs` is zero, in every build.
/// Constant time, apart from that panic.
impl RemAssign<u32> for PaddedBigInt {
    fn rem_assign(&mut self, rhs: u32) {
        assert!(
            rhs != 0,
            "attempt to calculate the remainder with a divisor of zero"
        );
        let limbs = self.limbs_mut();
        let sign = sign_fill(limbs);
        conditional_negate(limbs, sign);
        let remainder = div_assign_word(limbs, Word::from(rhs));
        limbs.fill(Limb::new(0));
        if let Some(low) = limbs.first_mut() {
            *low = Limb::new(remainder);
        }
        conditional_negate(limbs, sign);
    }
}

/// In the storage of `self`, as `/=`. Constant time, apart from the panic when
/// `rhs` is zero.
impl Div<u32> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn div(mut self, rhs: u32) -> PaddedBigInt {
        self /= rhs;
        self
    }
}

/// In a copy of `self`. Constant time, apart from the panic when `rhs` is zero.
impl Div<u32> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn div(self, rhs: u32) -> PaddedBigInt {
        self.clone() / rhs
    }
}

/// In the storage of `self`, as `%=`. Constant time, apart from the panic when
/// `rhs` is zero.
impl Rem<u32> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn rem(mut self, rhs: u32) -> PaddedBigInt {
        self %= rhs;
        self
    }
}

/// In a copy of `self`. Constant time, apart from the panic when `rhs` is zero.
impl Rem<u32> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn rem(self, rhs: u32) -> PaddedBigInt {
        self.clone() % rhs
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;

    use crate::{PaddedBigInt, Word};

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
    fn quotients_and_remainders_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES.into_iter().filter(|&b| b != 0) {
                let (Some(quotient), Some(remainder)) = (a.checked_div(b), a.checked_rem(b)) else {
                    continue;
                };
                let (x, y) = (PaddedBigInt::from(a), PaddedBigInt::from(b));
                let (quotient, remainder) =
                    (PaddedBigInt::from(quotient), PaddedBigInt::from(remainder));
                assert_eq!(&x / &y, quotient, "{a} {b}");
                assert_eq!(&x % &y, remainder, "{a} {b}");
                assert_eq!(x.div_rem(&y), (quotient, remainder), "{a} {b}");
            }
        }
    }

    #[test]
    fn division_truncates_toward_zero() {
        assert_eq!(
            PaddedBigInt::from(-7i8) / PaddedBigInt::from(2i8),
            PaddedBigInt::from(-3i8)
        );
        assert_eq!(
            PaddedBigInt::from(-7i8) % PaddedBigInt::from(2i8),
            PaddedBigInt::from(-1i8)
        );
        assert_eq!(
            PaddedBigInt::from(7i8) % PaddedBigInt::from(-2i8),
            PaddedBigInt::from(1i8)
        );
    }

    #[test]
    fn a_narrower_operand_is_sign_extended_to_the_wider_width() {
        let (quotient, remainder) = PaddedBigInt::from(-200i16).div_rem(&PaddedBigInt::from(7i128));
        assert_eq!(
            (&quotient, &remainder),
            (&PaddedBigInt::from(-28i8), &PaddedBigInt::from(-4i8))
        );
        let wide = (i128::BITS / Word::BITS) as usize;
        assert_eq!(
            (quotient.as_limbs().len(), remainder.as_limbs().len()),
            (wide, wide)
        );
    }

    #[test]
    fn the_most_negative_value_by_minus_one_leaves_no_remainder() {
        assert!((PaddedBigInt::from(i128::MIN) % PaddedBigInt::from(-1i8)).is_zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide with overflow")]
    fn the_most_negative_value_divided_by_minus_one_panics() {
        let _ = PaddedBigInt::from(i128::MIN) / PaddedBigInt::from(-1i8);
    }

    #[test]
    fn all_six_forms_of_each_give_the_same_result() {
        let (x, y) = (PaddedBigInt::from(-0xf0f0i128), PaddedBigInt::from(255i128));
        let (quotient, remainder) = (
            PaddedBigInt::from(-0xf0f0i128 / 255),
            PaddedBigInt::from(-0xf0f0i128 % 255),
        );
        assert_eq!(x.clone() / y.clone(), quotient);
        assert_eq!(x.clone() / &y, quotient);
        assert_eq!(&x / y.clone(), quotient);
        assert_eq!(&x / &y, quotient);
        let mut owned = x.clone();
        owned /= y.clone();
        assert_eq!(owned, quotient);
        let mut borrowed = x.clone();
        borrowed /= &y;
        assert_eq!(borrowed, quotient);
        assert_eq!(x.clone() % y.clone(), remainder);
        assert_eq!(x.clone() % &y, remainder);
        assert_eq!(&x % y.clone(), remainder);
        assert_eq!(&x % &y, remainder);
        let mut owned = x.clone();
        owned %= y.clone();
        assert_eq!(owned, remainder);
        let mut borrowed = x;
        borrowed %= &y;
        assert_eq!(borrowed, remainder);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn dividing_by_zero_panics() {
        let _ = PaddedBigInt::from(1i8) / PaddedBigInt::from(0i8);
    }

    #[test]
    #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
    fn a_remainder_by_zero_panics() {
        let _ = PaddedBigInt::from(1i8) % PaddedBigInt::from(0i8);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_rem_by_zero_panics() {
        let _ = PaddedBigInt::from(1i8).div_rem(&PaddedBigInt::from(0i8));
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
        fn dividing_by_a_word_matches_the_primitive_quotient_and_remainder() {
            for a in VALUES {
                for b in WORDS.into_iter().filter(|&b| b != 0) {
                    let (x, divisor) = (PaddedBigInt::from(a), i128::from(b));
                    assert_eq!(&x / b, PaddedBigInt::from(a / divisor), "{a} {b}");
                    assert_eq!(&x % b, PaddedBigInt::from(a % divisor), "{a} {b}");
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to divide by zero")]
        fn dividing_by_a_zero_word_panics() {
            let _ = PaddedBigInt::from(1i8) / 0u32;
        }

        #[test]
        #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
        fn the_remainder_by_a_zero_word_panics() {
            let _ = PaddedBigInt::from(1i8) % 0u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (PaddedBigInt::from(0xf0f0_f0f0i128), 255u32);
            let quotient = PaddedBigInt::from(0xf0f0_f0f0i128 / 255);
            let remainder = PaddedBigInt::from(0xf0f0_f0f0i128 % 255);
            assert_eq!(x.clone() / y, quotient);
            assert_eq!(&x / y, quotient);
            assert_eq!(x.clone() % y, remainder);
            assert_eq!(&x % y, remainder);
            let (mut divided, mut reduced) = (x.clone(), x.clone());
            divided /= y;
            reduced %= y;
            assert_eq!((divided, reduced), (quotient, remainder));
        }
    }
}
