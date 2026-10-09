//! Division and remainder of [`FixedBigUint`].

use core::ops::{Div, DivAssign, Rem, RemAssign};

use num_traits::Zero;

use super::FixedBigUint;
use crate::limb::{div_assign_word, div_rem_limbs};
use crate::wipe::replace_wiped;
use crate::{Limb, Word};
use tc_zeroize::Zeroize;

impl<const N: usize> FixedBigUint<N> {
    /// The quotient and the remainder by `rhs` together, for the work
    /// of one division. Panics when `rhs` is zero, in every build.
    /// Constant time, apart from that panic.
    pub fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        let mut quotient = self.clone();
        let remainder = quotient.divide(rhs);
        (quotient, remainder)
    }

    /// Replaces `self` with its quotient by `rhs`, which is not zero,
    /// and returns the remainder, by long division a bit at a time.
    /// Constant time.
    pub(super) fn divide(&mut self, rhs: &Self) -> Self {
        let mut remainder = Self::zero();
        div_rem_limbs(self.limbs_mut(), rhs.as_limbs(), 0, remainder.limbs_mut());
        remainder
    }
}

/// The quotient in place, by long division a bit at a time. Panics when
/// `rhs` is zero, in every build. Constant time, apart from that panic.
impl<const N: usize> DivAssign<&FixedBigUint<N>> for FixedBigUint<N> {
    fn div_assign(&mut self, rhs: &FixedBigUint<N>) {
        assert!(!rhs.is_zero(), "attempt to divide by zero");
        // the remainder is not wanted, and is wiped
        self.divide(rhs).zeroize();
    }
}

/// The remainder in place, by long division a bit at a time. Panics when
/// `rhs` is zero, in every build. Constant time, apart from that panic.
impl<const N: usize> RemAssign<&FixedBigUint<N>> for FixedBigUint<N> {
    fn rem_assign(&mut self, rhs: &FixedBigUint<N>) {
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
impl<const N: usize> DivAssign<FixedBigUint<N>> for FixedBigUint<N> {
    fn div_assign(&mut self, rhs: FixedBigUint<N>) {
        *self /= &rhs;
    }
}

/// The quotient in the storage of `self`, as `/=`. Constant time, apart
/// from the panics.
impl<const N: usize> Div<&FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn div(mut self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self /= rhs;
        self
    }
}

/// The quotient in the storage of `self`, as `/=`. Constant time, apart
/// from the panics.
impl<const N: usize> Div<FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn div(mut self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        self /= &rhs;
        self
    }
}

/// The quotient in a copy of `self`, on the stack. Constant time, apart
/// from the panics.
impl<const N: usize> Div<FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn div(self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        self.clone() / &rhs
    }
}

/// The quotient in a copy of `self`, on the stack. Constant time, apart
/// from the panics.
impl<const N: usize> Div<&FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn div(self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self.clone() / rhs
    }
}

/// The same as `%= &rhs`. Constant time, apart from the panics.
impl<const N: usize> RemAssign<FixedBigUint<N>> for FixedBigUint<N> {
    fn rem_assign(&mut self, rhs: FixedBigUint<N>) {
        *self %= &rhs;
    }
}

/// The remainder in the storage of `self`, as `%=`. Constant time, apart
/// from the panics.
impl<const N: usize> Rem<&FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn rem(mut self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self %= rhs;
        self
    }
}

/// The remainder in the storage of `self`, as `%=`. Constant time, apart
/// from the panics.
impl<const N: usize> Rem<FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn rem(mut self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        self %= &rhs;
        self
    }
}

/// The remainder in a copy of `self`, on the stack. Constant time, apart
/// from the panics.
impl<const N: usize> Rem<FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn rem(self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        self.clone() % &rhs
    }
}

/// The remainder in a copy of `self`, on the stack. Constant time, apart
/// from the panics.
impl<const N: usize> Rem<&FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn rem(self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self.clone() % rhs
    }
}

/// The quotient in place, by long division a bit at a time, so that no
/// hardware division sees the value. Panics when `rhs` is zero, in every
/// build. Constant time, apart from that panic.
impl<const N: usize> DivAssign<u32> for FixedBigUint<N> {
    fn div_assign(&mut self, rhs: u32) {
        assert!(rhs != 0, "attempt to divide by zero");
        div_assign_word(self.limbs_mut(), Word::from(rhs));
    }
}

/// The remainder in place, by the same long division a bit at a time.
/// Panics when `rhs` is zero, in every build. Constant time, apart from
/// that panic.
impl<const N: usize> RemAssign<u32> for FixedBigUint<N> {
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
impl<const N: usize> Div<u32> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn div(mut self, rhs: u32) -> FixedBigUint<N> {
        self /= rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic when
/// `rhs` is zero.
impl<const N: usize> Div<u32> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn div(self, rhs: u32) -> FixedBigUint<N> {
        self.clone() / rhs
    }
}

/// In the storage of `self`, as `%=`. Constant time, apart from the panic when
/// `rhs` is zero.
impl<const N: usize> Rem<u32> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn rem(mut self, rhs: u32) -> FixedBigUint<N> {
        self %= rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic when
/// `rhs` is zero.
impl<const N: usize> Rem<u32> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn rem(self, rhs: u32) -> FixedBigUint<N> {
        self.clone() % rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::{FixedBigUint, Limb, LimbArray, Word};

    /// The limbs of 128 bits, to compare against `u128`.
    const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

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
                let (x, y) = (
                    FixedBigUint::<LIMBS>::from(a),
                    FixedBigUint::<LIMBS>::from(b),
                );
                let (quotient, remainder) = (
                    FixedBigUint::<LIMBS>::from(a / b),
                    FixedBigUint::<LIMBS>::from(a % b),
                );
                assert_eq!(&x / &y, quotient, "{a} {b}");
                assert_eq!(&x % &y, remainder, "{a} {b}");
                assert_eq!(x.div_rem(&y), (quotient, remainder), "{a} {b}");
            }
        }
    }

    #[test]
    fn long_values_divide_back_to_the_dividend() {
        let mut state: u64 = 1;
        let mut value = || {
            let limbs = core::array::from_fn(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                Limb::new(state as Word)
            });
            FixedBigUint::<8>::new(LimbArray::new(limbs))
        };
        for shift in [0, Word::BITS + 3, 5 * Word::BITS + 1, 8 * Word::BITS - 1] {
            // never zero, as the low bit is set
            let (x, y) = (value(), (value() >> shift) | FixedBigUint::<8>::from(1u8));
            let (quotient, remainder) = x.div_rem(&y);
            assert_eq!(&quotient * &y + &remainder, x, "{shift}");
            assert!(remainder < y, "{shift}");
        }
    }

    #[test]
    fn all_six_forms_of_each_give_the_same_result() {
        let (x, y) = (
            FixedBigUint::<LIMBS>::from(0xf0f0u16),
            FixedBigUint::<LIMBS>::from(255u8),
        );
        let (quotient, remainder) = (
            FixedBigUint::<LIMBS>::from(0xf0f0u16 / 255),
            FixedBigUint::<LIMBS>::from(0xf0f0u16 % 255),
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
        let _ = FixedBigUint::<LIMBS>::from(1u8) / FixedBigUint::<LIMBS>::from(0u8);
    }

    #[test]
    #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
    fn a_remainder_by_zero_panics() {
        let _ = FixedBigUint::<LIMBS>::from(1u8) % FixedBigUint::<LIMBS>::from(0u8);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_rem_by_zero_panics() {
        let _ = FixedBigUint::<LIMBS>::from(1u8).div_rem(&FixedBigUint::<LIMBS>::from(0u8));
    }

    mod words {
        use crate::{FixedBigUint, Word};

        /// The limbs of 128 bits, to compare against `u128`.
        const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

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
                    let (x, divisor) = (FixedBigUint::<LIMBS>::from(a), u128::from(b));
                    assert_eq!(&x / b, FixedBigUint::<LIMBS>::from(a / divisor), "{a} {b}");
                    assert_eq!(&x % b, FixedBigUint::<LIMBS>::from(a % divisor), "{a} {b}");
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to divide by zero")]
        fn dividing_by_a_zero_word_panics() {
            let _ = FixedBigUint::<LIMBS>::from(1u8) / 0u32;
        }

        #[test]
        #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
        fn the_remainder_by_a_zero_word_panics() {
            let _ = FixedBigUint::<LIMBS>::from(1u8) % 0u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (FixedBigUint::<LIMBS>::from(0xf0f0_f0f0u128), 255u32);
            let quotient = FixedBigUint::<LIMBS>::from(0xf0f0_f0f0u128 / 255);
            let remainder = FixedBigUint::<LIMBS>::from(0xf0f0_f0f0u128 % 255);
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
