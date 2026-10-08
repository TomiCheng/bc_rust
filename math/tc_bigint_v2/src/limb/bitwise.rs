//! Bitwise operations over limbs.

use super::{Limb, Word};

/// Applies `op` to `a` and `b` limb by limb, in place in `a`, reading `b`
/// past its end as `b_fill`. Constant time: the lengths only decide how far
/// the loop runs.
pub(crate) fn bitwise_assign(
    a: &mut [Limb],
    b: &[Limb],
    b_fill: Word,
    op: impl Fn(Word, Word) -> Word,
) {
    for (index, limb) in a.iter_mut().enumerate() {
        let y = b.get(index).map_or(b_fill, |limb| limb.to_word());
        *limb = Limb::new(op(limb.to_word(), y));
    }
}

/// Inverts every bit of `limbs` in place. Constant time.
pub(crate) fn invert(limbs: &mut [Limb]) {
    for limb in limbs {
        *limb = Limb::new(!limb.to_word());
    }
}
