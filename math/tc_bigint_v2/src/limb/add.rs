//! Addition over limbs.

use super::{Limb, Word};

/// Adds `b`, read past its end as `b_fill`, into `a` in place and returns
/// the carry out of the top limb of `a`. Constant time: the lengths only
/// decide how far the loop runs.
pub(crate) fn add_assign_limbs(a: &mut [Limb], b: &[Limb], b_fill: Word) -> Word {
    let mut carry: Word = 0;
    for (index, limb) in a.iter_mut().enumerate() {
        let y = b.get(index).map_or(b_fill, |limb| limb.to_word());
        let (sum, first) = limb.to_word().overflowing_add(y);
        let (sum, second) = sum.overflowing_add(carry);
        *limb = Limb::new(sum);
        carry = Word::from(first | second);
    }
    carry
}

/// Whether adding two's-complement operands whose signs are `a_sign` and
/// `b_sign` overflowed into a result whose sign is `sum_sign`: it did when
/// the operands share a sign the result lacks. Each sign is all ones or
/// zero. Constant time.
pub(crate) fn signed_add_overflowed(a_sign: Word, b_sign: Word, sum_sign: Word) -> bool {
    (a_sign ^ sum_sign) & (b_sign ^ sum_sign) != 0
}
