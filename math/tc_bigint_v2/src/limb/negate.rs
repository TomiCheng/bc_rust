//! Two's-complement negation over limbs.

use super::{Limb, Word};

/// Negates the two's-complement `limbs` when `mask` is all ones and leaves
/// them as they are when it is zero. Constant time.
pub(crate) fn conditional_negate(limbs: &mut [Limb], mask: Word) {
    // negation is inverting every bit and adding one
    let mut carry = mask & 1;
    for limb in limbs {
        let (word, overflow) = (limb.to_word() ^ mask).overflowing_add(carry);
        *limb = Limb::new(word);
        carry = Word::from(overflow);
    }
}

/// The words of `b`, negated as two's complement when `mask` is all ones,
/// for reading a negative operand by its magnitude without copying it.
/// Constant time.
pub(crate) fn conditionally_negated(b: &[Limb], mask: Word) -> impl Iterator<Item = Word> + '_ {
    // negation is inverting every bit and adding one
    let mut carry = mask & 1;
    b.iter().map(move |limb| {
        let (word, overflow) = (limb.to_word() ^ mask).overflowing_add(carry);
        carry = Word::from(overflow);
        word
    })
}
