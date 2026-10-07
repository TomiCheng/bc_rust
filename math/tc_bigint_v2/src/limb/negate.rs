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
