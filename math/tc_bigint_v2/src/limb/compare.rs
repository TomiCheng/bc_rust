//! Comparing values held in limb slices of possibly different lengths.

use tc_constant_time::{Choice, ConstantTimeEq, ConstantTimeOrd};

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

/// Whether `a` is less than `b`, each read past its end as its `fill`; with
/// `signed`, as two's complement. Constant time: the lengths only decide how
/// far the loop runs.
pub(crate) fn ct_lt_extended(
    a: &[Limb],
    a_fill: Word,
    b: &[Limb],
    b_fill: Word,
    signed: bool,
) -> Choice {
    let len = a.len().max(b.len());
    let mut less = Choice::from_lsb(0);
    // from the lowest limb up, so the highest limb that differs decides
    for index in 0..len {
        let mut x = a.get(index).map_or(a_fill, |limb| limb.to_word());
        let mut y = b.get(index).map_or(b_fill, |limb| limb.to_word());
        if signed && index == len - 1 {
            // flipping the sign bit turns two's-complement order into unsigned order
            x ^= 1 << (Word::BITS - 1);
            y ^= 1 << (Word::BITS - 1);
        }
        let equal = x.ct_eq(&y);
        less = (equal & less) | (!equal & x.ct_lt(&y));
    }
    less
}
