//! The greatest common divisor of limbs, in constant time.

use super::{
    Limb, Word, ct_lt_extended, shl_assign_limbs_secret, shr_assign_limbs, shr_assign_limbs_secret,
};

/// Puts the greatest common divisor of unsigned `a` and `b`, which are as
/// long as each other, into `a`, using `b` for room; `gcd(0, 0)` is zero.
/// Constant time: binary GCD in a fixed number of rounds, two for each bit
/// of the length, every step done by masks.
pub(crate) fn gcd_assign_limbs(a: &mut [Limb], b: &mut [Limb]) {
    debug_assert_eq!(a.len(), b.len());
    // take out the power of two both share, to put back at the end
    let shared = trailing_zeros_of_either(a, b);
    shr_assign_limbs_secret(a, shared);
    shr_assign_limbs_secret(b, shared);
    // at most one of them is even now, unless both are zero: keep b odd
    conditional_swap(a, b, (low_bit(b) ^ 1).wrapping_neg());
    // With b odd, halving an even a, or taking b out of an odd a that is
    // the larger, keeps the divisor, and every round takes a bit out of a
    // or b, so after two rounds for each bit a is zero.
    for _ in 0..2 * a.len() * Word::BITS as usize {
        let odd = low_bit(a).wrapping_neg();
        let smaller = Word::from(ct_lt_extended(a, 0, b, 0, false).unwrap_u8()).wrapping_neg();
        conditional_swap(a, b, odd & smaller);
        sub_masked(a, b, odd);
        shr_assign_limbs(a, 1, 0);
    }
    // b holds the divisor without the power of two both shared
    shl_assign_limbs_secret(b, shared);
    a.copy_from_slice(b);
}

/// The lowest bit of `limbs`, zero for none.
fn low_bit(limbs: &[Limb]) -> Word {
    limbs.first().map_or(0, |limb| limb.to_word() & 1)
}

/// The zeros below the lowest bit set in either `a` or `b`, or zero when
/// neither has one. Constant time: every limb is looked at.
fn trailing_zeros_of_either(a: &[Limb], b: &[Limb]) -> u32 {
    let mut zeros: u32 = 0;
    // from the top down, so that the lowest limb with a set bit decides
    for (index, (x, y)) in a.iter().zip(b).enumerate().rev() {
        let word = x.to_word() | y.to_word();
        let here = index as u32 * Word::BITS + word.trailing_zeros();
        let set = 0u32.wrapping_sub(u32::from(word != 0));
        zeros = here & set | zeros & !set;
    }
    zeros
}

/// Swaps `a` and `b` limb by limb where `mask` is all ones. Constant time.
fn conditional_swap(a: &mut [Limb], b: &mut [Limb], mask: Word) {
    for (x, y) in a.iter_mut().zip(b.iter_mut()) {
        let difference = (x.to_word() ^ y.to_word()) & mask;
        *x = Limb::new(x.to_word() ^ difference);
        *y = Limb::new(y.to_word() ^ difference);
    }
}

/// Subtracts `b` from `a` where `mask` is all ones, and nothing where it is
/// zero, dropping the borrow. Constant time.
fn sub_masked(a: &mut [Limb], b: &[Limb], mask: Word) {
    let mut borrow: Word = 0;
    for (x, y) in a.iter_mut().zip(b) {
        let (difference, first) = x.to_word().overflowing_sub(y.to_word() & mask);
        let (difference, second) = difference.overflowing_sub(borrow);
        *x = Limb::new(difference);
        borrow = Word::from(first | second);
    }
}
