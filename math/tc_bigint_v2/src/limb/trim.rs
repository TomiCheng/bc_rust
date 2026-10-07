//! The shortest form of a value held in limbs.
//!
//! The length found follows the value, so these are variable time.

use super::{Limb, Word};

/// The length of unsigned `limbs` without their leading zero limbs.
pub(crate) fn trimmed_len_unsigned(limbs: &[Limb]) -> usize {
    limbs
        .iter()
        .rposition(|limb| limb.to_word() != 0)
        .map_or(0, |top| top + 1)
}

/// The length of two's-complement `limbs` without each top limb that only
/// repeats the sign bit of the limb below it; a lone zero limb goes too.
pub(crate) fn trimmed_len_signed(limbs: &[Limb]) -> usize {
    let mut len = limbs.len();
    while len >= 2 {
        let below_negative = limbs[len - 2].to_word() >> (Word::BITS - 1) == 1;
        let top = limbs[len - 1].to_word();
        if !((top == 0 && !below_negative) || (top == Word::MAX && below_negative)) {
            break;
        }
        len -= 1;
    }
    match limbs[..len] {
        [only] if only.to_word() == 0 => 0,
        _ => len,
    }
}
