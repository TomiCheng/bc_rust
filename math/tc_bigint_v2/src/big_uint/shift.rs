//! Bit shifts of [`BigUint`].

use alloc::vec::Vec;
use core::ops::{Shl, ShlAssign, Shr, ShrAssign};

use super::BigUint;
use crate::limb::{shl_assign_limbs, shr_assign_limbs};
use crate::{Limb, Word};

/// In the storage of `self`, which grows by exactly the limbs the result
/// needs, so it never overflows; zero stays zero. Variable time: only for
/// public values.
impl ShlAssign<u32> for BigUint {
    fn shl_assign(&mut self, shift: u32) {
        let Some(top) = self.as_limbs().last().map(|limb| limb.to_word()) else {
            return;
        };
        // whole limbs, and one more when the top limb has fewer free bits
        // than the rest of the shift
        let grow =
            (shift / Word::BITS) as usize + usize::from(top.leading_zeros() < shift % Word::BITS);
        let mut limbs = core::mem::take(self).into_limbs();
        limbs.resize(limbs.len() + grow, Limb::new(0));
        shl_assign_limbs(&mut limbs, shift);
        *self = BigUint::new(limbs);
    }
}

/// In the storage of `self`, as `<<=`. Variable time: only for public
/// values.
impl Shl<u32> for BigUint {
    type Output = BigUint;

    fn shl(mut self, shift: u32) -> BigUint {
        self <<= shift;
        self
    }
}

/// In a copy of `self` with room for the result, so it allocates once.
/// Variable time: only for public values.
impl Shl<u32> for &BigUint {
    type Output = BigUint;

    fn shl(self, shift: u32) -> BigUint {
        let room = self.as_limbs().len() + (shift / Word::BITS) as usize + 1;
        let mut limbs = Vec::with_capacity(room);
        limbs.extend_from_slice(self.as_limbs());
        BigUint::new(limbs) << shift
    }
}

/// In the storage of `self`: bits shifted past the bottom are dropped, and
/// the result is trimmed. Variable time: only for public values.
impl ShrAssign<u32> for BigUint {
    fn shr_assign(&mut self, shift: u32) {
        let mut limbs = core::mem::take(self).into_limbs();
        shr_assign_limbs(&mut limbs, shift, 0);
        *self = BigUint::new(limbs);
    }
}

/// In the storage of `self`, as `>>=`. Variable time: only for public
/// values.
impl Shr<u32> for BigUint {
    type Output = BigUint;

    fn shr(mut self, shift: u32) -> BigUint {
        self >>= shift;
        self
    }
}

/// In a copy of `self`. Variable time: only for public values.
impl Shr<u32> for &BigUint {
    type Output = BigUint;

    fn shr(self, shift: u32) -> BigUint {
        self.clone() >> shift
    }
}

#[cfg(test)]
mod tests {
    use crate::{BigUint, Word};

    const VALUES: [u128; 7] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX,
    ];

    const SHIFTS: [u32; 9] = [
        0,
        1,
        7,
        Word::BITS - 1,
        Word::BITS,
        Word::BITS + 1,
        100,
        127,
        200,
    ];

    #[test]
    fn shifting_left_doubles_once_per_bit() {
        for a in VALUES {
            for shift in SHIFTS {
                let x = BigUint::from(a);
                let mut expected = x.clone();
                for _ in 0..shift {
                    expected = &expected + &expected;
                }
                assert_eq!(&x << shift, expected, "{a} {shift}");
            }
        }
    }

    #[test]
    fn shifting_right_matches_the_primitive_ones() {
        for a in VALUES {
            for shift in SHIFTS {
                let expected = a.checked_shr(shift).unwrap_or(0);
                assert_eq!(
                    &BigUint::from(a) >> shift,
                    BigUint::from(expected),
                    "{a} {shift}"
                );
            }
        }
    }

    #[test]
    fn shifting_zero_left_leaves_no_limbs() {
        assert!((BigUint::default() << 200).as_limbs().is_empty());
    }

    #[test]
    fn all_three_forms_of_each_shift_give_the_same_result() {
        let x = BigUint::from(0xf0f0u128);
        let left = BigUint::from(0xf0f0u128 << 9);
        assert_eq!(x.clone() << 9, left);
        assert_eq!(&x << 9, left);
        let mut assigned = x.clone();
        assigned <<= 9;
        assert_eq!(assigned, left);
        let right = BigUint::from(0xf0f0u128 >> 9);
        assert_eq!(x.clone() >> 9, right);
        assert_eq!(&x >> 9, right);
        let mut assigned = x;
        assigned >>= 9;
        assert_eq!(assigned, right);
    }
}
