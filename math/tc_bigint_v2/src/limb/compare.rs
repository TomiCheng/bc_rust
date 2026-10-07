//! Comparing values held in limb slices of possibly different lengths.

use tc_constant_time::{Choice, ConstantTimeEq};

use super::{Limb, Word};

/// Whether `a` and `b` hold the same value, each read past its end as its
/// `fill`: zero for unsigned values, the sign for two's complement.
/// Constant time: the lengths, which are public, only decide how far the
/// loop runs.
pub(crate) fn ct_eq_extended(a: &[Limb], a_fill: Word, b: &[Limb], b_fill: Word) -> Choice {
    let mut difference: Word = 0;
    for index in 0..a.len().max(b.len()) {
        let x = a.get(index).map_or(a_fill, |limb| limb.to_word());
        let y = b.get(index).map_or(b_fill, |limb| limb.to_word());
        difference |= x ^ y;
    }
    difference.ct_eq(&0)
}
