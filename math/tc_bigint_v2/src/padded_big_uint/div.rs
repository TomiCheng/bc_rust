//! Division and remainder of [`PaddedBigUint`].

use alloc::vec;
use core::ops::{Div, DivAssign, Rem, RemAssign};

use num_traits::Zero;

use super::PaddedBigUint;
use crate::limb::{div_assign_word, div_rem_limbs};
use crate::wipe::replace_wiped;
use crate::{Limb, Word};
use tc_zeroize::Zeroize;

impl PaddedBigUint {
    /// The quotient and the remainder by `rhs` together, for the work
    /// of one division, at the wider width. Panics when `rhs` is zero,
    /// in every build. Constant time, apart from that panic.
    pub fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        let mut quotient = self.clone_for(rhs);
        let remainder = quotient.divide(rhs);
        (quotient, remainder)
    }

    /// Replaces `self` with its quotient by `rhs`, which is not zero,
    /// at the wider width, and returns the remainder at that width,
    /// which a new buffer holds; by long division a bit at a time.
    /// Constant time.
    pub(super) fn divide(&mut self, rhs: &Self) -> Self {
        self.widen(rhs.as_limbs().len());
        let mut remainder = Self::new(vec![Limb::new(0); self.as_limbs().len()].into_boxed_slice());
        div_rem_limbs(self.limbs_mut(), rhs.as_limbs(), 0, remainder.limbs_mut());
        remainder
    }
}

/// The quotient in place at the wider width: the narrower operand is
/// extended to it with zeros, and the result takes it. The remainder is
/// worked out in a new buffer, so it allocates once. Panics when `rhs` is
/// zero, in every build. Constant time, apart from that panic.
impl DivAssign<&PaddedBigUint> for PaddedBigUint {
    fn div_assign(&mut self, rhs: &PaddedBigUint) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        // the remainder is not wanted, and is wiped
        self.divide(rhs).zeroize();
    }
}

/// The remainder at the wider width, in the new buffer it is worked out in.
/// Panics when `rhs` is zero, in every build. Constant time, apart from
/// that panic.
impl RemAssign<&PaddedBigUint> for PaddedBigUint {
    fn rem_assign(&mut self, rhs: &PaddedBigUint) {
        assert!(
            !rhs.is_zero(),
            "attempt to calculate the remainder with a divisor of zero"
        );
        let remainder = self.divide(rhs);
        // the quotient it takes the place of is not wanted, and is wiped
        replace_wiped(self, remainder);
    }
}

/// The same as `/= &rhs`. Constant time, apart from the panics.
impl DivAssign<PaddedBigUint> for PaddedBigUint {
    fn div_assign(&mut self, rhs: PaddedBigUint) {
        *self /= &rhs;
    }
}

/// The quotient in the storage of `self`, as `/=`. Constant time, apart
/// from the panics.
impl Div<&PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn div(mut self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self /= rhs;
        self
    }
}

/// The quotient in the storage of `self`, as `/=`. Constant time, apart
/// from the panics.
impl Div<PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn div(mut self, rhs: PaddedBigUint) -> PaddedBigUint {
        self /= &rhs;
        self
    }
}

/// The quotient in a copy of `self` at the wider width. Constant time,
/// apart from the panics.
impl Div<PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn div(self, rhs: PaddedBigUint) -> PaddedBigUint {
        self.clone_for(&rhs) / &rhs
    }
}

/// The quotient in a copy of `self` at the wider width. Constant time,
/// apart from the panics.
impl Div<&PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn div(self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self.clone_for(rhs) / rhs
    }
}

/// The same as `%= &rhs`. Constant time, apart from the panics.
impl RemAssign<PaddedBigUint> for PaddedBigUint {
    fn rem_assign(&mut self, rhs: PaddedBigUint) {
        *self %= &rhs;
    }
}

/// The remainder in the storage of `self`, as `%=`. Constant time, apart
/// from the panics.
impl Rem<&PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn rem(mut self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self %= rhs;
        self
    }
}

/// The remainder in the storage of `self`, as `%=`. Constant time, apart
/// from the panics.
impl Rem<PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn rem(mut self, rhs: PaddedBigUint) -> PaddedBigUint {
        self %= &rhs;
        self
    }
}

/// The remainder in a copy of `self` at the wider width. Constant time,
/// apart from the panics.
impl Rem<PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn rem(self, rhs: PaddedBigUint) -> PaddedBigUint {
        self.clone_for(&rhs) % &rhs
    }
}

/// The remainder in a copy of `self` at the wider width. Constant time,
/// apart from the panics.
impl Rem<&PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn rem(self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self.clone_for(rhs) % rhs
    }
}

/// The quotient in place at the width of `self`, which the word does not
/// widen, by long division a bit at a time, so that no hardware division
/// sees the value. Panics when `rhs` is zero, in every build. Constant
/// time, apart from that panic.
impl DivAssign<u32> for PaddedBigUint {
    fn div_assign(&mut self, rhs: u32) {
        assert!(rhs != 0, "attempt to divide by zero");
        div_assign_word(self.limbs_mut(), Word::from(rhs));
    }
}

/// The remainder in place at the width of `self`, which the word does not
/// widen, by the same long division a bit at a time. Panics when `rhs` is
/// zero, in every build. Constant time, apart from that panic.
impl RemAssign<u32> for PaddedBigUint {
    fn rem_assign(&mut self, rhs: u32) {
        assert!(
            rhs != 0,
            "attempt to calculate the remainder with a divisor of zero"
        );
        let limbs = self.limbs_mut();
        let remainder = div_assign_word(limbs, Word::from(rhs));
        limbs.fill(Limb::new(0));
        if let Some(low) = limbs.first_mut() {
            *low = Limb::new(remainder);
        }
    }
}

/// In the storage of `self`, as `/=`. Constant time, apart from the panic when
/// `rhs` is zero.
impl Div<u32> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn div(mut self, rhs: u32) -> PaddedBigUint {
        self /= rhs;
        self
    }
}

/// In a copy of `self`. Constant time, apart from the panic when `rhs` is zero.
impl Div<u32> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn div(self, rhs: u32) -> PaddedBigUint {
        self.clone() / rhs
    }
}

/// In the storage of `self`, as `%=`. Constant time, apart from the panic when
/// `rhs` is zero.
impl Rem<u32> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn rem(mut self, rhs: u32) -> PaddedBigUint {
        self %= rhs;
        self
    }
}

/// In a copy of `self`. Constant time, apart from the panic when `rhs` is zero.
impl Rem<u32> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn rem(self, rhs: u32) -> PaddedBigUint {
        self.clone() % rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::{PaddedBigUint, Word};

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
    fn quotients_and_remainders_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES.into_iter().filter(|&b| b != 0) {
                let (x, y) = (PaddedBigUint::from(a), PaddedBigUint::from(b));
                let (quotient, remainder) =
                    (PaddedBigUint::from(a / b), PaddedBigUint::from(a % b));
                assert_eq!(&x / &y, quotient, "{a} {b}");
                assert_eq!(&x % &y, remainder, "{a} {b}");
                assert_eq!(x.div_rem(&y), (quotient, remainder), "{a} {b}");
            }
        }
    }

    #[test]
    fn a_narrower_operand_is_zero_extended_to_the_wider_width() {
        let (quotient, remainder) = PaddedBigUint::from(200u8).div_rem(&PaddedBigUint::from(7u128));
        assert_eq!(
            (&quotient, &remainder),
            (&PaddedBigUint::from(28u8), &PaddedBigUint::from(4u8))
        );
        let wide = (u128::BITS / Word::BITS) as usize;
        assert_eq!(
            (quotient.as_limbs().len(), remainder.as_limbs().len()),
            (wide, wide)
        );
    }

    #[test]
    fn all_six_forms_of_each_give_the_same_result() {
        let (x, y) = (
            PaddedBigUint::from(0xf0f0u128),
            PaddedBigUint::from(255u128),
        );
        let (quotient, remainder) = (
            PaddedBigUint::from(0xf0f0u128 / 255),
            PaddedBigUint::from(0xf0f0u128 % 255),
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
        let _ = PaddedBigUint::from(1u8) / PaddedBigUint::from(0u8);
    }

    #[test]
    #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
    fn a_remainder_by_zero_panics() {
        let _ = PaddedBigUint::from(1u8) % PaddedBigUint::from(0u8);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_rem_by_zero_panics() {
        let _ = PaddedBigUint::from(1u8).div_rem(&PaddedBigUint::from(0u8));
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
        fn dividing_by_a_word_matches_the_primitive_quotient_and_remainder() {
            for a in VALUES {
                for b in WORDS.into_iter().filter(|&b| b != 0) {
                    let (x, divisor) = (PaddedBigUint::from(a), u128::from(b));
                    assert_eq!(&x / b, PaddedBigUint::from(a / divisor), "{a} {b}");
                    assert_eq!(&x % b, PaddedBigUint::from(a % divisor), "{a} {b}");
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to divide by zero")]
        fn dividing_by_a_zero_word_panics() {
            let _ = PaddedBigUint::from(1u8) / 0u32;
        }

        #[test]
        #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
        fn the_remainder_by_a_zero_word_panics() {
            let _ = PaddedBigUint::from(1u8) % 0u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (PaddedBigUint::from(0xf0f0_f0f0u128), 255u32);
            let quotient = PaddedBigUint::from(0xf0f0_f0f0u128 / 255);
            let remainder = PaddedBigUint::from(0xf0f0_f0f0u128 % 255);
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
