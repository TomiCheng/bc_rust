//! Shifting the bits held in limbs.

use super::{Limb, Word};

/// Shifts `limbs` left by `shift` bits within their length: zeros come in
/// at the bottom, and bits shifted past the top are dropped. Constant time
/// in the value; `shift` decides which limbs move where, so it must be
/// public.
pub(crate) fn shl_assign_limbs(limbs: &mut [Limb], shift: u32) {
    shl_masked(limbs, shift, Word::MAX);
}

/// Shifts `limbs` right by `shift` bits within their length: `fill` comes
/// in at the top, zero for unsigned values and the sign for two's
/// complement, and bits shifted past the bottom are dropped. Constant time
/// in the value; `shift` decides which limbs move where, so it must be
/// public.
pub(crate) fn shr_assign_limbs(limbs: &mut [Limb], shift: u32, fill: Word) {
    shr_masked(limbs, shift, fill, Word::MAX);
}

/// Shifts unsigned `limbs` left by `shift` bits, which must be below their
/// width, keeping `shift` secret too: one stage for each power of two below
/// the width, each shifting by it or not by a mask. Constant time.
pub(crate) fn shl_assign_limbs_secret(limbs: &mut [Limb], shift: u32) {
    for stage in stages(limbs.len()) {
        shl_masked(
            limbs,
            1 << stage,
            Word::from(shift >> stage & 1 == 1).wrapping_neg(),
        );
    }
}

/// Shifts unsigned `limbs` right by `shift` bits, which must be below their
/// width, keeping `shift` secret too, as [`shl_assign_limbs_secret`] does.
/// Constant time.
pub(crate) fn shr_assign_limbs_secret(limbs: &mut [Limb], shift: u32) {
    for stage in stages(limbs.len()) {
        shr_masked(
            limbs,
            1 << stage,
            0,
            Word::from(shift >> stage & 1 == 1).wrapping_neg(),
        );
    }
}

/// The exponents of the powers of two below the width of `len` limbs.
fn stages(len: usize) -> impl Iterator<Item = u32> {
    let bits = len as u32 * Word::BITS;
    (0..u32::BITS).take_while(move |&stage| 1 << stage < bits)
}

/// [`shl_assign_limbs`] where `mask` is all ones; where it is zero, every
/// limb is written back as it was.
fn shl_masked(limbs: &mut [Limb], shift: u32, mask: Word) {
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
        limbs[index] = Limb::new(word & mask | limbs[index].to_word() & !mask);
    }
}

/// [`shr_assign_limbs`] where `mask` is all ones; where it is zero, every
/// limb is written back as it was.
fn shr_masked(limbs: &mut [Limb], shift: u32, fill: Word, mask: Word) {
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
        limbs[index] = Limb::new(word & mask | limbs[index].to_word() & !mask);
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

#[cfg(test)]
mod tests {
    use super::{
        shl_assign_limbs, shl_assign_limbs_secret, shr_assign_limbs, shr_assign_limbs_secret,
    };
    use crate::{Limb, Word};

    #[test]
    fn secret_shifts_match_the_public_ones_for_every_shift_below_the_width() {
        let value: [Limb; 3] = core::array::from_fn(|index| {
            Limb::new((0x0123_4567_89ab_cdefu64 as Word).rotate_left(index as u32 * 7))
        });
        for shift in 0..3 * Word::BITS {
            let (mut public, mut secret) = (value, value);
            shl_assign_limbs(&mut public, shift);
            shl_assign_limbs_secret(&mut secret, shift);
            assert_eq!(public, secret, "{shift}");
            let (mut public, mut secret) = (value, value);
            shr_assign_limbs(&mut public, shift, 0);
            shr_assign_limbs_secret(&mut secret, shift);
            assert_eq!(public, secret, "{shift}");
        }
    }
}
