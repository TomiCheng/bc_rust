//! Modular addition and subtraction on limbs, for values below the modulus
//! and of its width.

use tc_bigint_v2::{Limb, Word};

/// `value + rhs mod modulus`, into `value`. The sum may carry out of the
/// width, and the one subtraction of the modulus that follows takes that
/// carry into account. Constant time.
pub(crate) fn add_mod_assign(value: &mut [Limb], rhs: &[Limb], modulus: &[Limb]) {
    let carry = add_masked(value, rhs, Word::MAX);
    reduce_once(value, carry, modulus);
}

/// `value - rhs mod modulus`, never negative, into `value`. When the
/// difference borrows, it has wrapped around the width, and adding the
/// modulus wraps it back into range. Constant time.
pub(crate) fn sub_mod_assign(value: &mut [Limb], rhs: &[Limb], modulus: &[Limb]) {
    let borrow = sub_masked(value, rhs, Word::MAX);
    add_masked(value, modulus, borrow.wrapping_neg());
}

/// Takes `modulus` from `value`, whose word above its limbs is `top`, zero
/// or one, when the two together are not below it; a value below twice the
/// modulus ends below it. Constant time.
pub(super) fn reduce_once(value: &mut [Limb], top: Word, modulus: &[Limb]) {
    let below = borrow_out(value, modulus);
    sub_masked(value, modulus, (top | (below ^ 1)).wrapping_neg());
}

/// Adds `addend & mask` to `value` limb by limb, and returns the carry out.
/// Constant time.
fn add_masked(value: &mut [Limb], addend: &[Limb], mask: Word) -> Word {
    let mut carry: Word = 0;
    for (limb, other) in value.iter_mut().zip(addend) {
        let (sum, over) = limb.to_word().overflowing_add(other.to_word() & mask);
        let (sum, again) = sum.overflowing_add(carry);
        *limb = Limb::new(sum);
        carry = Word::from(over | again);
    }
    carry
}

/// Takes `subtrahend & mask` from `value` limb by limb, and returns the
/// borrow out. Constant time.
fn sub_masked(value: &mut [Limb], subtrahend: &[Limb], mask: Word) -> Word {
    let mut borrow: Word = 0;
    for (limb, other) in value.iter_mut().zip(subtrahend) {
        let (difference, under) = limb.to_word().overflowing_sub(other.to_word() & mask);
        let (difference, again) = difference.overflowing_sub(borrow);
        *limb = Limb::new(difference);
        borrow = Word::from(under | again);
    }
    borrow
}

/// The borrow out of `value - other`, one when `value` is below `other`,
/// without writing the difference. Constant time.
fn borrow_out(value: &[Limb], other: &[Limb]) -> Word {
    let mut borrow: Word = 0;
    for (limb, other) in value.iter().zip(other) {
        let (difference, under) = limb.to_word().overflowing_sub(other.to_word());
        let (_, again) = difference.overflowing_sub(borrow);
        borrow = Word::from(under | again);
    }
    borrow
}
