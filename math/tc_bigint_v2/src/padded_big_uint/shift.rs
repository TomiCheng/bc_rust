//! Bit shifts of [`PaddedBigUint`].

use core::ops::{Shl, ShlAssign, Shr, ShrAssign};

use super::PaddedBigUint;
use crate::Word;
use crate::limb::{shl_assign_limbs, shr_assign_limbs};

/// In place at the width: bits shifted past the top are dropped. Panics
/// when `shift` is not below the width, in every build, unlike the
/// primitive integers, whose check depends on the profile. Constant time in
/// the value; `shift` must be public.
impl ShlAssign<u32> for PaddedBigUint {
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
impl Shl<u32> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn shl(mut self, shift: u32) -> PaddedBigUint {
        self <<= shift;
        self
    }
}

/// In a copy of `self`, so it allocates once. Constant time in the value;
/// `shift` must be public.
impl Shl<u32> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn shl(self, shift: u32) -> PaddedBigUint {
        self.clone() << shift
    }
}

/// In place at the width, zeros coming in at the top. Panics when `shift`
/// is not below the width, in every build, unlike the primitive integers,
/// whose check depends on the profile. Constant time in the value; `shift`
/// must be public.
impl ShrAssign<u32> for PaddedBigUint {
    fn shr_assign(&mut self, shift: u32) {
        assert!(
            (shift as usize) < self.as_limbs().len() * Word::BITS as usize,
            "attempt to shift right with overflow"
        );
        shr_assign_limbs(self.limbs_mut(), shift, 0);
    }
}

/// In the storage of `self`, as `>>=`. Constant time in the value; `shift`
/// must be public.
impl Shr<u32> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn shr(mut self, shift: u32) -> PaddedBigUint {
        self >>= shift;
        self
    }
}

/// In a copy of `self`, so it allocates once. Constant time in the value;
/// `shift` must be public.
impl Shr<u32> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn shr(self, shift: u32) -> PaddedBigUint {
        self.clone() >> shift
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
                let x = PaddedBigUint::from(a);
                assert_eq!(&x << shift, PaddedBigUint::from(a << shift), "{a} {shift}");
                assert_eq!(&x >> shift, PaddedBigUint::from(a >> shift), "{a} {shift}");
            }
        }
    }

    #[test]
    fn a_shift_keeps_the_width() {
        let shifted = PaddedBigUint::from(1u128) << 127;
        assert_eq!(shifted.as_limbs().len(), (u128::BITS / Word::BITS) as usize);
    }

    #[test]
    fn all_three_forms_of_each_shift_give_the_same_result() {
        let x = PaddedBigUint::from(0xf0f0u128);
        let left = PaddedBigUint::from(0xf0f0u128 << 9);
        assert_eq!(x.clone() << 9, left);
        assert_eq!(&x << 9, left);
        let mut assigned = x.clone();
        assigned <<= 9;
        assert_eq!(assigned, left);
        let right = PaddedBigUint::from(0xf0f0u128 >> 9);
        assert_eq!(x.clone() >> 9, right);
        assert_eq!(&x >> 9, right);
        let mut assigned = x;
        assigned >>= 9;
        assert_eq!(assigned, right);
    }

    #[test]
    #[should_panic(expected = "attempt to shift left with overflow")]
    fn shifting_left_by_the_width_panics() {
        let _ = PaddedBigUint::from(1u8) << Word::BITS;
    }

    #[test]
    #[should_panic(expected = "attempt to shift right with overflow")]
    fn shifting_right_by_the_width_panics() {
        let _ = PaddedBigUint::from(1u8) >> Word::BITS;
    }
}
