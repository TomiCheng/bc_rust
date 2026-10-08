//! Subtraction over limbs.

use super::{Limb, Word};

/// Subtracts `b`, read past its end as `b_fill`, from `a` in place and
/// returns the borrow out of the top limb of `a`. Constant time: the
/// lengths only decide how far the loop runs.
pub(crate) fn sub_assign_limbs(a: &mut [Limb], b: &[Limb], b_fill: Word) -> Word {
    let mut borrow: Word = 0;
    for (index, limb) in a.iter_mut().enumerate() {
        let y = b.get(index).map_or(b_fill, |limb| limb.to_word());
        let (difference, first) = limb.to_word().overflowing_sub(y);
        let (difference, second) = difference.overflowing_sub(borrow);
        *limb = Limb::new(difference);
        borrow = Word::from(first | second);
    }
    borrow
}

/// Whether subtracting two's-complement operands whose signs are
/// `minuend_sign` and `subtrahend_sign` overflowed into a result whose sign
/// is `difference_sign`: it did when the operands' signs differ and the
/// result lacks the sign of the minuend. Each sign is all ones or zero.
/// Constant time.
pub(crate) fn signed_sub_overflowed(
    minuend_sign: Word,
    subtrahend_sign: Word,
    difference_sign: Word,
) -> bool {
    (minuend_sign ^ subtrahend_sign) & (minuend_sign ^ difference_sign) != 0
}
