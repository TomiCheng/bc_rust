//! Putting a bound in place of a result that overflowed, without a branch.

use super::{Limb, Word};

/// Replaces every limb of unsigned `limbs` with `fill` when `overflowed`:
/// all ones for the largest value, zero for the smallest. Constant time.
pub(crate) fn saturate_unsigned(limbs: &mut [Limb], overflowed: bool, fill: Word) {
    let mask = Word::from(overflowed).wrapping_neg();
    for limb in limbs {
        *limb = Limb::new(limb.to_word() & !mask | fill & mask);
    }
}

/// Replaces two's-complement `limbs` when `overflowed` with the largest
/// value when `sign` is zero and the smallest when it is all ones. Constant
/// time: the position of the top limb is public.
pub(crate) fn saturate_signed(limbs: &mut [Limb], overflowed: bool, sign: Word) {
    let mask = Word::from(overflowed).wrapping_neg();
    let top = limbs.len().saturating_sub(1);
    for (index, limb) in limbs.iter_mut().enumerate() {
        // all ones below the top limb and its sign bit clear for the
        // largest value; the bit pattern flipped for the smallest
        let bound = match index == top {
            true => (Word::MAX >> 1) ^ sign,
            false => !sign,
        };
        *limb = Limb::new(limb.to_word() & !mask | bound & mask);
    }
}
