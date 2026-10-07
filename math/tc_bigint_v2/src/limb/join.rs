//! Reading a primitive value back out of limbs.
//!
//! Whether a value fits decides the result, so these are variable time.

use super::{Limb, Word};
use crate::encoding::sign_fill;

const PER_VALUE: usize = (u128::BITS / Word::BITS) as usize;

/// The low 128 bits of `limbs`, reading `fill` above them.
fn low_u128(limbs: &[Limb], fill: Word) -> u128 {
    (0..PER_VALUE).fold(0, |value, index| {
        let word = limbs.get(index).map_or(fill, |limb| limb.to_word());
        value | (word as u128) << (index as u32 * Word::BITS)
    })
}

/// Whether every limb above the low 128 bits equals `fill`.
fn high_limbs_are(limbs: &[Limb], fill: Word) -> bool {
    limbs
        .iter()
        .skip(PER_VALUE)
        .all(|limb| limb.to_word() == fill)
}

/// Unsigned `limbs` as a `u128`, if they fit.
pub(crate) fn unsigned_to_u128(limbs: &[Limb]) -> Option<u128> {
    high_limbs_are(limbs, 0).then(|| low_u128(limbs, 0))
}

/// Two's-complement `limbs` as an `i128`, if they fit.
pub(crate) fn signed_to_i128(limbs: &[Limb]) -> Option<i128> {
    let value = low_u128(limbs, sign_fill(limbs)) as i128;
    // the limbs above must repeat the sign the low 128 bits end with
    let fill = match value < 0 {
        true => Word::MAX,
        false => 0,
    };
    high_limbs_are(limbs, fill).then_some(value)
}

/// Two's-complement `limbs` as a `u128`, if they are not negative and fit.
pub(crate) fn signed_to_u128(limbs: &[Limb]) -> Option<u128> {
    (sign_fill(limbs) == 0)
        .then(|| unsigned_to_u128(limbs))
        .flatten()
}
