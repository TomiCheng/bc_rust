//! Modular addition and subtraction on limbs, for values below the modulus
//! and of its width.

use tc_bigint_v2::{Limb, Word};

use crate::limb::{add_masked, reduce_once, sub_masked};

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
