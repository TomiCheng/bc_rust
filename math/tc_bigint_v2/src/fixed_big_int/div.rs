//! Division and remainder of [`FixedBigInt`].

use core::ops::{Div, DivAssign, Rem, RemAssign};

use num_traits::Zero;

use super::FixedBigInt;
use crate::encoding::sign_fill;
use crate::limb::{conditional_negate, div_assign_word, signed_div_rem_limbs};
use crate::wipe::replace_wiped;
use crate::{Limb, Word};
use tc_zeroize::Zeroize;

impl<const N: usize> FixedBigInt<N> {
    /// The quotient and the remainder by `rhs` together, for the work
    /// of one division, the quotient truncated toward zero. Panics when
    /// `rhs` is zero, and when the quotient does not fit, which only
    /// the most negative value divided by -1 does, in every build.
    /// Constant time, apart from those panics.
    pub fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        let mut quotient = self.clone();
        let (remainder, overflowed) = quotient.divide(rhs);
        assert!(!overflowed, "attempt to divide with overflow");
        (quotient, remainder)
    }

    /// Replaces `self` with its quotient by `rhs`, which is not zero,
    /// truncated toward zero, and returns the remainder, which takes
    /// the sign of `self`, and whether the quotient overflowed.
    /// Constant time.
    pub(super) fn divide(&mut self, rhs: &Self) -> (Self, bool) {
        let mut remainder = Self::zero();
        let overflowed =
            signed_div_rem_limbs(self.limbs_mut(), rhs.as_limbs(), remainder.limbs_mut());
        (remainder, overflowed)
    }
}

/// The quotient in place, truncated toward zero, by long division a bit at
/// a time on the magnitudes. Panics when `rhs` is zero, and when the
/// quotient does not fit, which only the most negative value divided by -1
/// does, in every build. Constant time, apart from those panics.
impl<const N: usize> DivAssign<&FixedBigInt<N>> for FixedBigInt<N> {
    fn div_assign(&mut self, rhs: &FixedBigInt<N>) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        let (mut remainder, overflowed) = self.divide(rhs);
        // the remainder is not wanted, and is wiped
        remainder.zeroize();
        assert!(!overflowed, "attempt to divide with overflow");
    }
}

/// The remainder in place, with the sign of `self`. Panics when `rhs` is
/// zero, in every build; the most negative value by -1 gives zero, where
/// the primitive integers panic, as the remainder itself always fits.
/// Constant time, apart from that panic.
impl<const N: usize> RemAssign<&FixedBigInt<N>> for FixedBigInt<N> {
    fn rem_assign(&mut self, rhs: &FixedBigInt<N>) {
        assert!(
            !rhs.is_zero(),
            "attempt to calculate the remainder with a divisor of zero"
        );
        let (remainder, _) = self.divide(rhs);
        // the quotient it takes the place of is not wanted, and is wiped
        replace_wiped(self, remainder);
    }
}

/// The same as `/= &rhs`. Constant time, apart from the panics.
impl<const N: usize> DivAssign<FixedBigInt<N>> for FixedBigInt<N> {
    fn div_assign(&mut self, rhs: FixedBigInt<N>) {
        *self /= &rhs;
    }
}

/// The quotient in the storage of `self`, as `/=`. Constant time, apart
/// from the panics.
impl<const N: usize> Div<&FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn div(mut self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self /= rhs;
        self
    }
}

/// The quotient in the storage of `self`, as `/=`. Constant time, apart
/// from the panics.
impl<const N: usize> Div<FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn div(mut self, rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        self /= &rhs;
        self
    }
}

/// The quotient in a copy of `self`, on the stack. Constant time, apart
/// from the panics.
impl<const N: usize> Div<FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn div(self, rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        self.clone() / &rhs
    }
}

/// The quotient in a copy of `self`, on the stack. Constant time, apart
/// from the panics.
impl<const N: usize> Div<&FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn div(self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self.clone() / rhs
    }
}

/// The same as `%= &rhs`. Constant time, apart from the panics.
impl<const N: usize> RemAssign<FixedBigInt<N>> for FixedBigInt<N> {
    fn rem_assign(&mut self, rhs: FixedBigInt<N>) {
        *self %= &rhs;
    }
}

/// The remainder in the storage of `self`, as `%=`. Constant time, apart
/// from the panics.
impl<const N: usize> Rem<&FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn rem(mut self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self %= rhs;
        self
    }
}

/// The remainder in the storage of `self`, as `%=`. Constant time, apart
/// from the panics.
impl<const N: usize> Rem<FixedBigInt<N>> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn rem(mut self, rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        self %= &rhs;
        self
    }
}

/// The remainder in a copy of `self`, on the stack. Constant time, apart
/// from the panics.
impl<const N: usize> Rem<FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn rem(self, rhs: FixedBigInt<N>) -> FixedBigInt<N> {
        self.clone() % &rhs
    }
}

/// The remainder in a copy of `self`, on the stack. Constant time, apart
/// from the panics.
impl<const N: usize> Rem<&FixedBigInt<N>> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn rem(self, rhs: &FixedBigInt<N>) -> FixedBigInt<N> {
        self.clone() % rhs
    }
}

/// The quotient, truncated toward zero, in place: the magnitude is divided
/// by long division a bit at a time, so that no hardware division sees it,
/// then the sign is put back; it cannot overflow, as the word is positive.
/// Panics when `rhs` is zero, in every build. Constant time, apart from
/// that panic.
impl<const N: usize> DivAssign<u32> for FixedBigInt<N> {
    fn div_assign(&mut self, rhs: u32) {
        assert!(rhs != 0, "attempt to divide by zero");
        let limbs = self.limbs_mut();
        let sign = sign_fill(limbs);
        conditional_negate(limbs, sign);
        div_assign_word(limbs, Word::from(rhs));
        conditional_negate(limbs, sign);
    }
}

/// The remainder, with the sign of `self`, in place, by the same long
/// division of the magnitude a bit at a time. Panics when `rhs` is zero, in
/// every build. Constant time, apart from that panic.
impl<const N: usize> RemAssign<u32> for FixedBigInt<N> {
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
impl<const N: usize> Div<u32> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn div(mut self, rhs: u32) -> FixedBigInt<N> {
        self /= rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic when
/// `rhs` is zero.
impl<const N: usize> Div<u32> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn div(self, rhs: u32) -> FixedBigInt<N> {
        self.clone() / rhs
    }
}

/// In the storage of `self`, as `%=`. Constant time, apart from the panic when
/// `rhs` is zero.
impl<const N: usize> Rem<u32> for FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn rem(mut self, rhs: u32) -> FixedBigInt<N> {
        self %= rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic when
/// `rhs` is zero.
impl<const N: usize> Rem<u32> for &FixedBigInt<N> {
    type Output = FixedBigInt<N>;

    fn rem(self, rhs: u32) -> FixedBigInt<N> {
        self.clone() % rhs
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;

    use crate::{FixedBigInt, Word};

    /// The limbs of 128 bits, to compare against `i128`.
    const LIMBS: usize = (i128::BITS / Word::BITS) as usize;

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
                let (x, y) = (FixedBigInt::<LIMBS>::from(a), FixedBigInt::<LIMBS>::from(b));
                let (quotient, remainder) = (
                    FixedBigInt::<LIMBS>::from(quotient),
                    FixedBigInt::<LIMBS>::from(remainder),
                );
                assert_eq!(&x / &y, quotient, "{a} {b}");
                assert_eq!(&x % &y, remainder, "{a} {b}");
                assert_eq!(x.div_rem(&y), (quotient, remainder), "{a} {b}");
            }
        }
    }

    #[test]
    fn division_truncates_toward_zero() {
        assert_eq!(
            FixedBigInt::<LIMBS>::from(-7i8) / FixedBigInt::<LIMBS>::from(2i8),
            FixedBigInt::<LIMBS>::from(-3i8)
        );
        assert_eq!(
            FixedBigInt::<LIMBS>::from(-7i8) % FixedBigInt::<LIMBS>::from(2i8),
            FixedBigInt::<LIMBS>::from(-1i8)
        );
        assert_eq!(
            FixedBigInt::<LIMBS>::from(7i8) % FixedBigInt::<LIMBS>::from(-2i8),
            FixedBigInt::<LIMBS>::from(1i8)
        );
    }

    #[test]
    fn the_most_negative_value_by_minus_one_leaves_no_remainder() {
        let min = FixedBigInt::<LIMBS>::from(i128::MIN);
        assert!((min % FixedBigInt::from(-1i8)).is_zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide with overflow")]
    fn the_most_negative_value_divided_by_minus_one_panics() {
        let _ = FixedBigInt::<LIMBS>::from(i128::MIN) / FixedBigInt::from(-1i8);
    }

    #[test]
    fn all_six_forms_of_each_give_the_same_result() {
        let (x, y) = (
            FixedBigInt::<LIMBS>::from(-0xf0f0i32),
            FixedBigInt::<LIMBS>::from(255i16),
        );
        let (quotient, remainder) = (
            FixedBigInt::<LIMBS>::from(-0xf0f0i32 / 255),
            FixedBigInt::<LIMBS>::from(-0xf0f0i32 % 255),
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
        let _ = FixedBigInt::<LIMBS>::from(1i8) / FixedBigInt::<LIMBS>::from(0i8);
    }

    #[test]
    #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
    fn a_remainder_by_zero_panics() {
        let _ = FixedBigInt::<LIMBS>::from(1i8) % FixedBigInt::<LIMBS>::from(0i8);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_rem_by_zero_panics() {
        let _ = FixedBigInt::<LIMBS>::from(1i8).div_rem(&FixedBigInt::<LIMBS>::from(0i8));
    }

    mod words {
        use crate::{FixedBigInt, Word};

        /// The limbs of 128 bits, to compare against `i128`.
        const LIMBS: usize = (i128::BITS / Word::BITS) as usize;

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
                    let (x, divisor) = (FixedBigInt::<LIMBS>::from(a), i128::from(b));
                    assert_eq!(&x / b, FixedBigInt::<LIMBS>::from(a / divisor), "{a} {b}");
                    assert_eq!(&x % b, FixedBigInt::<LIMBS>::from(a % divisor), "{a} {b}");
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to divide by zero")]
        fn dividing_by_a_zero_word_panics() {
            let _ = FixedBigInt::<LIMBS>::from(1u8) / 0u32;
        }

        #[test]
        #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
        fn the_remainder_by_a_zero_word_panics() {
            let _ = FixedBigInt::<LIMBS>::from(1u8) % 0u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (FixedBigInt::<LIMBS>::from(0xf0f0_f0f0i128), 255u32);
            let quotient = FixedBigInt::<LIMBS>::from(0xf0f0_f0f0i128 / 255);
            let remainder = FixedBigInt::<LIMBS>::from(0xf0f0_f0f0i128 % 255);
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
