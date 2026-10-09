//! Random limbs, drawn from a generator the caller gives.

use rand_core::TryRng;
use tc_zeroize::Zeroize;

use super::{Limb, Word};

/// Fills the low `bits` bits of `limbs` with bytes of `rng`, low first, and
/// clears the bits above; `limbs` must hold `bits` bits. It draws the bytes
/// those bits take and no more, so a seeded generator gives the same value
/// on every target. Constant time: `bits` must be public.
pub(crate) fn fill_bits<R: TryRng + ?Sized>(
    limbs: &mut [Limb],
    bits: u32,
    rng: &mut R,
) -> Result<(), R::Error> {
    let mut remaining = bits;
    for limb in limbs.iter_mut() {
        let taken = remaining.min(Word::BITS);
        let mut bytes = [0u8; size_of::<Word>()];
        let drawn = rng.try_fill_bytes(&mut bytes[..taken.div_ceil(8) as usize]);
        // the low `taken` bits of the word, which are all the bytes give
        // but the top bits of the last one
        let mask = if taken == Word::BITS {
            Word::MAX
        } else {
            (1 << taken) - 1
        };
        *limb = Limb::new(Word::from_le_bytes(bytes) & mask);
        bytes.zeroize();
        drawn?;
        remaining -= taken;
    }
    Ok(())
}
