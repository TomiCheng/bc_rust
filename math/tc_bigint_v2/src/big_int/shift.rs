//! Bit shifts of [`BigInt`].

use alloc::vec::Vec;
use core::ops::{Shl, ShlAssign, Shr, ShrAssign};

use super::BigInt;
use crate::encoding::sign_fill;
use crate::limb::{shl_assign_limbs, shr_assign_limbs};
use crate::{Limb, Word};

/// In the storage of `self`, which grows by exactly the limbs the result
/// needs, so it never overflows; zero stays zero. Variable time: only for
/// public values.
impl ShlAssign<u32> for BigInt {
    fn shl_assign(&mut self, shift: u32) {
        let Some(top) = self.as_limbs().last().map(|limb| limb.to_word()) else {
            return;
        };
        let fill = sign_fill(self.as_limbs());
        // the bits of the top limb above its sign bit that only repeat it
        let free = match fill {
            0 => top.leading_zeros(),
            _ => top.leading_ones(),
        } - 1;
        // whole limbs, and one more when the top limb has fewer free bits
        // than the rest of the shift
        let grow = (shift / Word::BITS) as usize + usize::from(free < shift % Word::BITS);
        let mut limbs = core::mem::take(self).into_limbs();
        limbs.resize(limbs.len() + grow, Limb::new(fill));
        shl_assign_limbs(&mut limbs, shift);
        *self = BigInt::new(limbs);
    }
}

/// In the storage of `self`, as `<<=`. Variable time: only for public
/// values.
impl Shl<u32> for BigInt {
    type Output = BigInt;

    fn shl(mut self, shift: u32) -> BigInt {
        self <<= shift;
        self
    }
}

/// In a copy of `self` with room for the result, so it allocates once.
/// Variable time: only for public values.
impl Shl<u32> for &BigInt {
    type Output = BigInt;

    fn shl(self, shift: u32) -> BigInt {
        let room = self.as_limbs().len() + (shift / Word::BITS) as usize + 1;
        let mut limbs = Vec::with_capacity(room);
        limbs.extend_from_slice(self.as_limbs());
        BigInt::new(limbs) << shift
    }
}

/// In the storage of `self`, the sign coming in at the top, which rounds
/// toward negative infinity; the result is trimmed. Variable time: only for
/// public values.
impl ShrAssign<u32> for BigInt {
    fn shr_assign(&mut self, shift: u32) {
        let mut limbs = core::mem::take(self).into_limbs();
        let fill = sign_fill(&limbs);
        shr_assign_limbs(&mut limbs, shift, fill);
        *self = BigInt::new(limbs);
    }
}

/// In the storage of `self`, as `>>=`. Variable time: only for public
/// values.
impl Shr<u32> for BigInt {
    type Output = BigInt;

    fn shr(mut self, shift: u32) -> BigInt {
        self >>= shift;
        self
    }
}

/// In a copy of `self`. Variable time: only for public values.
impl Shr<u32> for &BigInt {
    type Output = BigInt;

    fn shr(self, shift: u32) -> BigInt {
        self.clone() >> shift
    }
}

#[cfg(test)]
mod tests {
    use crate::{BigInt, Word};

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
                let x = BigInt::from(a);
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
                let expected = a.checked_shr(shift).unwrap_or(a >> 127);
                assert_eq!(
                    &BigInt::from(a) >> shift,
                    BigInt::from(expected),
                    "{a} {shift}"
                );
            }
        }
    }

    #[test]
    fn shifting_right_rounds_toward_negative_infinity() {
        assert_eq!(BigInt::from(-1i8) >> 1, BigInt::from(-1i8));
        assert_eq!(BigInt::from(-3i8) >> 1, BigInt::from(-2i8));
    }

    #[test]
    fn shifting_zero_left_leaves_no_limbs() {
        assert!((BigInt::default() << 200).as_limbs().is_empty());
    }

    #[test]
    fn all_three_forms_of_each_shift_give_the_same_result() {
        let x = BigInt::from(-0xf0f0i128);
        let left = BigInt::from(-0xf0f0i128 << 9);
        assert_eq!(x.clone() << 9, left);
        assert_eq!(&x << 9, left);
        let mut assigned = x.clone();
        assigned <<= 9;
        assert_eq!(assigned, left);
        let right = BigInt::from(-0xf0f0i128 >> 9);
        assert_eq!(x.clone() >> 9, right);
        assert_eq!(&x >> 9, right);
        let mut assigned = x;
        assigned >>= 9;
        assert_eq!(assigned, right);
    }
}
