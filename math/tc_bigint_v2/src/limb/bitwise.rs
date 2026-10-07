//! Bitwise operations over limbs.

use super::{Limb, Word};

/// Fills `out` with `op` of `a` and `b` limb by limb, each read past its end
/// as its `fill`. Constant time: the lengths only decide how far the loop
/// runs.
pub(crate) fn bitwise_into(
    a: &[Limb],
    a_fill: Word,
    b: &[Limb],
    b_fill: Word,
    out: &mut [Limb],
    op: impl Fn(Word, Word) -> Word,
) {
    for (index, limb) in out.iter_mut().enumerate() {
        let x = a.get(index).map_or(a_fill, |limb| limb.to_word());
        let y = b.get(index).map_or(b_fill, |limb| limb.to_word());
        *limb = Limb::new(op(x, y));
    }
}
