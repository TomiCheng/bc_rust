//! Bit shifts of [`PaddedBigInt`].

use core::ops::{Shl, ShlAssign, Shr, ShrAssign};

use super::PaddedBigInt;
use crate::Word;
use crate::encoding::sign_fill;
use crate::limb::{shl_assign_limbs, shr_assign_limbs};

/// In place at the width: bits shifted past the top are dropped, so the
/// sign may change, as for the primitive integers. Panics when `shift` is
/// not below the width, in every build, unlike the primitive integers,
/// whose check depends on the profile. Constant time in the value; `shift`
/// must be public.
impl ShlAssign<u32> for PaddedBigInt {
    fn shl_assign(&mut self, shift: u32) {
        assert!(
            (shift as usize) < self.as_limbs().len() * Word::BITS as usize,
            "attempt to shift left with overflow"
        );
        shl_assign_limbs(self.limbs_mut(), shift);
    }
}

/// In the storage of `self`, as `<<=`. Constant time in the value; `shift`
/// must be public.
impl Shl<u32> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn shl(mut self, shift: u32) -> PaddedBigInt {
        self <<= shift;
        self
    }
}

/// In a copy of `self`, so it allocates once. Constant time in the value;
/// `shift` must be public.
impl Shl<u32> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn shl(self, shift: u32) -> PaddedBigInt {
        self.clone() << shift
    }
}

/// In place at the width, the sign coming in at the top, which rounds
/// toward negative infinity. Panics when `shift` is not below the width, in
/// every build, unlike the primitive integers, whose check depends on the
/// profile. Constant time in the value; `shift` must be public.
impl ShrAssign<u32> for PaddedBigInt {
    fn shr_assign(&mut self, shift: u32) {
        assert!(
            (shift as usize) < self.as_limbs().len() * Word::BITS as usize,
            "attempt to shift right with overflow"
        );
        let fill = sign_fill(self.as_limbs());
        shr_assign_limbs(self.limbs_mut(), shift, fill);
    }
}

/// In the storage of `self`, as `>>=`. Constant time in the value; `shift`
/// must be public.
impl Shr<u32> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn shr(mut self, shift: u32) -> PaddedBigInt {
        self >>= shift;
        self
    }
}

/// In a copy of `self`, so it allocates once. Constant time in the value;
/// `shift` must be public.
impl Shr<u32> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn shr(self, shift: u32) -> PaddedBigInt {
        self.clone() >> shift
    }
}

#[cfg(test)]
mod tests {
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

    const SHIFTS: [u32; 8] = [
        0,
        1,
        7,
        Word::BITS - 1,
        Word::BITS,
        Word::BITS + 1,
        100,
        127,
    ];

    #[test]
    fn shifts_at_128_bits_match_the_primitive_ones() {
        for a in VALUES {
            for shift in SHIFTS {
                let x = PaddedBigInt::from(a);
                assert_eq!(&x << shift, PaddedBigInt::from(a << shift), "{a} {shift}");
                assert_eq!(&x >> shift, PaddedBigInt::from(a >> shift), "{a} {shift}");
            }
        }
    }

    #[test]
    fn a_shift_keeps_the_width() {
        let shifted = PaddedBigInt::from(-1i128) >> 127;
        assert_eq!(shifted.as_limbs().len(), (i128::BITS / Word::BITS) as usize);
    }

    #[test]
    fn all_three_forms_of_each_shift_give_the_same_result() {
        let x = PaddedBigInt::from(-0xf0f0i128);
        let left = PaddedBigInt::from(-0xf0f0i128 << 9);
        assert_eq!(x.clone() << 9, left);
        assert_eq!(&x << 9, left);
        let mut assigned = x.clone();
        assigned <<= 9;
        assert_eq!(assigned, left);
        let right = PaddedBigInt::from(-0xf0f0i128 >> 9);
        assert_eq!(x.clone() >> 9, right);
        assert_eq!(&x >> 9, right);
        let mut assigned = x;
        assigned >>= 9;
        assert_eq!(assigned, right);
    }

    #[test]
    #[should_panic(expected = "attempt to shift left with overflow")]
    fn shifting_left_by_the_width_panics() {
        let _ = PaddedBigInt::from(1i8) << Word::BITS;
    }

    #[test]
    #[should_panic(expected = "attempt to shift right with overflow")]
    fn shifting_right_by_the_width_panics() {
        let _ = PaddedBigInt::from(-1i8) >> Word::BITS;
    }
}
