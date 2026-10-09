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

/// `2 · value mod modulus`, into `value`, by shifting it up a bit: the bit
/// shifted out of the width is the word above it, and the one subtraction
/// of the modulus that follows takes it into account. Constant time.
pub(crate) fn double_mod_assign(value: &mut [Limb], modulus: &[Limb]) {
    let mut carry: Word = 0;
    for limb in value.iter_mut() {
        let word = limb.to_word();
        *limb = Limb::new((word << 1) | carry);
        carry = word >> (Word::BITS - 1);
    }
    reduce_once(value, carry, modulus);
}

/// `value - rhs mod modulus`, never negative, into `value`. When the
/// difference borrows, it has wrapped around the width, and adding the
/// modulus wraps it back into range. Constant time.
pub(crate) fn sub_mod_assign(value: &mut [Limb], rhs: &[Limb], modulus: &[Limb]) {
    let borrow = sub_masked(value, rhs, Word::MAX);
    add_masked(value, modulus, borrow.wrapping_neg());
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{Limb, Word};

    use super::{add_mod_assign, double_mod_assign};

    #[test]
    fn doubling_is_adding_to_itself_even_when_the_top_bit_shifts_out() {
        for m in [1, 3, 0xff, Word::MAX / 3, Word::MAX - 2, Word::MAX] {
            for value in [0, 1, m / 2, m - 1] {
                let modulus = [Limb::new(m)];
                let (mut doubled, mut added) = ([Limb::new(value % m)], [Limb::new(value % m)]);
                double_mod_assign(&mut doubled, &modulus);
                add_mod_assign(&mut added, &[Limb::new(value % m)], &modulus);
                assert_eq!(doubled, added, "{value} {m}");
            }
        }
    }
}
