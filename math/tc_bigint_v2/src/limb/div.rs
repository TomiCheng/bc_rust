//! Division over limbs, in place and in constant time.

use core::iter::repeat;

use super::{Limb, Word, conditional_negate, conditionally_negated};
use crate::encoding::sign_fill;

/// Divides `a` by `b` in place, as unsigned values, reading `b` negated
/// when `b_mask` is all ones, so that a negative `b` counts by its
/// magnitude: `a` becomes the quotient, and `remainder`, as long as `a` and
/// zero on entry, the remainder. `b` must not be zero. Constant time: one
/// round per bit of `a`, and the lengths only decide how far each round
/// runs.
pub(crate) fn div_rem_limbs(a: &mut [Limb], b: &[Limb], b_mask: Word, remainder: &mut [Limb]) {
    let bits = Word::BITS as usize;
    // Long division from the top bit down: each bit of the dividend moves
    // into the remainder, and the divisor is taken out of the remainder
    // whenever it fits, which is the quotient bit. The quotient bit goes
    // where the dividend bit was, so a holds the quotient above the bit
    // and the dividend below it.
    for bit in (0..a.len() * bits).rev() {
        let (index, shift) = (bit / bits, (bit % bits) as u32);
        let mut pushed_out = (a[index].to_word() >> shift) & 1;
        for limb in remainder.iter_mut() {
            let word = limb.to_word();
            *limb = Limb::new(word << 1 | pushed_out);
            pushed_out = word >> (Word::BITS - 1);
        }

        let divisor = || conditionally_negated(b, b_mask).chain(repeat(0));
        let mut borrow: Word = 0;
        for (limb, y) in remainder.iter_mut().zip(divisor()) {
            let (difference, first) = limb.to_word().overflowing_sub(y);
            let (difference, second) = difference.overflowing_sub(borrow);
            *limb = Limb::new(difference);
            borrow = Word::from(first | second);
        }
        // the divisor fitted when a bit was pushed out above the remainder
        // or the subtraction did not borrow; otherwise it is added back
        let fitted = pushed_out | (borrow ^ 1);
        let add_back = fitted.wrapping_sub(1);
        let mut carry: Word = 0;
        for (limb, y) in remainder.iter_mut().zip(divisor()) {
            let (sum, first) = limb.to_word().overflowing_add(y & add_back);
            let (sum, second) = sum.overflowing_add(carry);
            *limb = Limb::new(sum);
            carry = Word::from(first | second);
        }

        a[index] = Limb::new(a[index].to_word() & !(1 << shift) | fitted << shift);
    }
}

/// Divides two's-complement `a` by `b` in place, truncating toward zero:
/// `a` becomes the quotient, and `remainder`, as long as `a` and zero on
/// entry, the remainder, which takes the sign of `a`. Returns whether the
/// quotient did not fit, which only the most negative value divided by -1
/// does; the remainder always fits. `b` must not be zero. Constant time.
pub(crate) fn signed_div_rem_limbs(a: &mut [Limb], b: &[Limb], remainder: &mut [Limb]) -> bool {
    let (a_sign, b_sign) = (sign_fill(a), sign_fill(b));
    // divide the magnitudes, then give each result its sign
    conditional_negate(a, a_sign);
    div_rem_limbs(a, b, b_sign, remainder);
    let negative = a_sign ^ b_sign;
    conditional_negate(a, negative);
    conditional_negate(remainder, a_sign);
    // a quotient other than zero must come out with the sign it should have
    let nonzero = a.iter().fold(0, |any, limb| any | limb.to_word()) != 0;
    let wrong_sign = sign_fill(a) != negative;
    nonzero & wrong_sign
}
