//! Shifting the bits held in limbs.

use super::{Limb, Word};

/// Shifts `limbs` left by `shift` bits within their length: zeros come in
/// at the bottom, and bits shifted past the top are dropped. Constant time
/// in the value; `shift` decides which limbs move where, so it must be
/// public.
pub(crate) fn shl_assign_limbs(limbs: &mut [Limb], shift: u32) {
    let (limb_shift, bit_shift) = ((shift / Word::BITS) as usize, shift % Word::BITS);
    // from the top down, so that each limb is read before it is overwritten
    for index in (0..limbs.len()).rev() {
        let high = word_below(limbs, index, limb_shift);
        let word = match bit_shift {
            0 => high,
            _ => {
                let low = word_below(limbs, index, limb_shift + 1);
                (high << bit_shift) | (low >> (Word::BITS - bit_shift))
            }
        };
        limbs[index] = Limb::new(word);
    }
}

/// Shifts `limbs` right by `shift` bits within their length: `fill` comes
/// in at the top, zero for unsigned values and the sign for two's
/// complement, and bits shifted past the bottom are dropped. Constant time
/// in the value; `shift` decides which limbs move where, so it must be
/// public.
pub(crate) fn shr_assign_limbs(limbs: &mut [Limb], shift: u32, fill: Word) {
    let (limb_shift, bit_shift) = ((shift / Word::BITS) as usize, shift % Word::BITS);
    // from the bottom up, so that each limb is read before it is overwritten
    for index in 0..limbs.len() {
        let low = word_above(limbs, index, limb_shift, fill);
        let word = match bit_shift {
            0 => low,
            _ => {
                let high = word_above(limbs, index, limb_shift + 1, fill);
                (low >> bit_shift) | (high << (Word::BITS - bit_shift))
            }
        };
        limbs[index] = Limb::new(word);
    }
}

/// The word `distance` limbs below `index`, or zero below the bottom.
fn word_below(limbs: &[Limb], index: usize, distance: usize) -> Word {
    index
        .checked_sub(distance)
        .map_or(0, |source| limbs[source].to_word())
}

/// The word `distance` limbs above `index`, or `fill` above the top.
fn word_above(limbs: &[Limb], index: usize, distance: usize, fill: Word) -> Word {
    limbs
        .get(index + distance)
        .map_or(fill, |limb| limb.to_word())
}
