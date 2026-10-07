//! Splitting a primitive value into limbs.

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

use super::{Limb, Word};

/// Writes `value` into `out`, least significant limb first; limbs above
/// 128 bits get `fill`. Constant time.
pub(crate) fn split_u128_into(value: u128, fill: Word, out: &mut [Limb]) {
    let per_value = (u128::BITS / Word::BITS) as usize;
    for (index, limb) in out.iter_mut().enumerate() {
        *limb = Limb::new(if index < per_value {
            (value >> (index as u32 * Word::BITS)) as Word
        } else {
            fill
        });
    }
}

/// The low `bits` bits of `value` as limbs, least significant first; `bits`
/// is at most 128 and decides the limb count. Constant time.
#[cfg(feature = "alloc")]
pub(crate) fn split_u128(value: u128, bits: u32) -> Vec<Limb> {
    let mut limbs = vec![Limb::new(0); bits.div_ceil(Word::BITS) as usize];
    split_u128_into(value, 0, &mut limbs);
    limbs
}
