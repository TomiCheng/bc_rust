//! Finding, reading and changing single bits in limbs.

use super::{Limb, Word};

/// The bits up to and including the highest one of `limbs` that differs
/// from `fill`: the highest set bit of unsigned limbs with a zero fill, the
/// highest bit apart from the sign extension of two's complement with the
/// sign. Constant time: every limb is looked at.
pub(crate) fn bits_limbs(limbs: &[Limb], fill: Word) -> u32 {
    let mut bits: u32 = 0;
    for (index, limb) in limbs.iter().enumerate() {
        let word = limb.to_word() ^ fill;
        let here = index as u32 * Word::BITS + (Word::BITS - word.leading_zeros());
        // the highest limb that is not all fill decides
        let differs = 0u32.wrapping_sub(u32::from(word != 0));
        bits = here & differs | bits & !differs;
    }
    bits
}

/// The zeros below the lowest set bit of `limbs`, or `None` when they are
/// all zero. Constant time: every limb is looked at, and only the result
/// shows whether one was set.
pub(crate) fn trailing_zeros_limbs(limbs: &[Limb]) -> Option<u32> {
    let mut zeros: u32 = 0;
    let mut found: u32 = 0;
    // from the top down, so that the lowest limb with a set bit decides
    for (index, limb) in limbs.iter().enumerate().rev() {
        let word = limb.to_word();
        let here = index as u32 * Word::BITS + word.trailing_zeros();
        let set = 0u32.wrapping_sub(u32::from(word != 0));
        zeros = here & set | zeros & !set;
        found |= set;
    }
    (found != 0).then_some(zeros)
}

/// Bit `index` of `limbs`, read past their end as `fill`. Constant time:
/// `index`, which must be public, only decides which limb is read.
pub(crate) fn bit_at(limbs: &[Limb], index: u32, fill: Word) -> bool {
    let (limb, shift) = ((index / Word::BITS) as usize, index % Word::BITS);
    limbs.get(limb).map_or(fill, |limb| limb.to_word()) >> shift & 1 == 1
}

/// Sets bit `index` of `limbs`, which must hold it, to `value`. Constant
/// time in `value`; `index` must be public.
pub(crate) fn set_bit_at(limbs: &mut [Limb], index: u32, value: bool) {
    let (limb, shift) = ((index / Word::BITS) as usize, index % Word::BITS);
    let word = limbs[limb].to_word() & !(1 << shift) | Word::from(value) << shift;
    limbs[limb] = Limb::new(word);
}

/// Flips bit `index` of `limbs`, which must hold it. Constant time; `index`
/// must be public.
pub(crate) fn flip_bit_at(limbs: &mut [Limb], index: u32) {
    let (limb, shift) = ((index / Word::BITS) as usize, index % Word::BITS);
    limbs[limb] = Limb::new(limbs[limb].to_word() ^ 1 << shift);
}
