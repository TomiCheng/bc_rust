//! Multiplication over limbs, in place and in constant time.

use super::{Limb, WideWord, Word, conditional_negate, conditionally_negated};
use crate::encoding::sign_fill;

/// Multiplies `a` by `b` in place over the length of `a`, as unsigned
/// values, reading `b` negated when `b_mask` is all ones, so that a
/// negative `b` counts by its magnitude. Returns whether the product lost
/// bits past the top of `a`. Constant time: the lengths only decide how far
/// the loops run.
pub(crate) fn mul_assign_limbs(a: &mut [Limb], b: &[Limb], b_mask: Word) -> bool {
    let mut lost: Word = 0;
    // From the top down: a[i] is taken out and a[i] * b added at i and
    // above, where only limbs of the product so far are; the limbs below i
    // still hold the multiplicand.
    for i in (0..a.len()).rev() {
        let x = WideWord::from(a[i].to_word());
        a[i] = Limb::new(0);
        let mut carry: Word = 0;
        for (j, y) in conditionally_negated(b, b_mask).enumerate() {
            let current = a.get(i + j).map_or(0, |limb| limb.to_word());
            let t = x * WideWord::from(y) + WideWord::from(current) + WideWord::from(carry);
            match a.get_mut(i + j) {
                Some(limb) => *limb = Limb::new(t as Word),
                None => lost |= t as Word,
            }
            carry = (t >> Word::BITS) as Word;
        }
        // the carry runs up through the product so far
        for limb in a.iter_mut().skip(i + b.len()) {
            let (sum, overflow) = limb.to_word().overflowing_add(carry);
            *limb = Limb::new(sum);
            carry = Word::from(overflow);
        }
        lost |= carry;
    }
    lost != 0
}

/// Multiplies two's-complement `a` by `b` in place over the length of `a`
/// and returns whether the product did not fit. Constant time: the lengths
/// only decide how far the loops run.
pub(crate) fn signed_mul_assign_limbs(a: &mut [Limb], b: &[Limb]) -> bool {
    let (a_sign, b_sign) = (sign_fill(a), sign_fill(b));
    // the product of the magnitudes, then its sign
    conditional_negate(a, a_sign);
    let lost = mul_assign_limbs(a, b, b_sign);
    let negative = a_sign ^ b_sign;
    conditional_negate(a, negative);
    // a product other than zero must come out with the sign it should have
    let nonzero = a.iter().fold(0, |any, limb| any | limb.to_word()) != 0;
    let wrong_sign = sign_fill(a) != negative;
    lost | (nonzero & wrong_sign)
}
