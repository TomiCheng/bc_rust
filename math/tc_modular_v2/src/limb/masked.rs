//! Limb arithmetic steered by word masks, all ones or all zeros, so that
//! nothing branches on the values.

use tc_bigint_v2::{Limb, Word};

/// Adds `addend & mask` to `value` limb by limb, and returns the carry out.
/// Constant time.
pub(crate) fn add_masked(value: &mut [Limb], addend: &[Limb], mask: Word) -> Word {
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
pub(crate) fn sub_masked(value: &mut [Limb], subtrahend: &[Limb], mask: Word) -> Word {
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
pub(crate) fn borrow_out(value: &[Limb], other: &[Limb]) -> Word {
    let mut borrow: Word = 0;
    for (limb, other) in value.iter().zip(other) {
        let (difference, under) = limb.to_word().overflowing_sub(other.to_word());
        let (_, again) = difference.overflowing_sub(borrow);
        borrow = Word::from(under | again);
    }
    borrow
}

/// Takes `modulus` from `value`, whose word above its limbs is `top`, zero
/// or one, when the two together are not below it; a value below twice the
/// modulus ends below it. Constant time.
pub(crate) fn reduce_once(value: &mut [Limb], top: Word, modulus: &[Limb]) {
    let below = borrow_out(value, modulus);
    sub_masked(value, modulus, (top | (below ^ 1)).wrapping_neg());
}

/// Overwrites `value` with `source` where `mask` is all ones, and leaves it
/// where it is zero. Constant time.
pub(crate) fn select_assign(value: &mut [Limb], source: &[Limb], mask: Word) {
    for (limb, other) in value.iter_mut().zip(source) {
        *limb = Limb::new((other.to_word() & mask) | (limb.to_word() & !mask));
    }
}

/// Halves `value` in place where `mask` is all ones, with `top`, zero or
/// one, as the bit above its limbs, which comes down into the top limb; it
/// leaves `value` where `mask` is zero. Constant time: the width is public.
pub(crate) fn halve_masked(value: &mut [Limb], top: Word, mask: Word) {
    for index in 0..value.len() {
        let above = value.get(index + 1).map_or(top, |limb| limb.to_word());
        let word = value[index].to_word();
        let halved = (word >> 1) | (above << (Word::BITS - 1));
        value[index] = Limb::new((halved & mask) | (word & !mask));
    }
}

/// One when `value` is one, and zero otherwise. Constant time.
pub(crate) fn is_one(value: &[Limb]) -> Word {
    let mut differs = value.first().map_or(1, |limb| limb.to_word() ^ 1);
    for limb in value.iter().skip(1) {
        differs |= limb.to_word();
    }
    // Zero exactly when nothing differs from one.
    1 ^ ((differs | differs.wrapping_neg()) >> (Word::BITS - 1))
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{Limb, Word};

    use super::{halve_masked, is_one, select_assign};

    fn limbs(words: [Word; 2]) -> [Limb; 2] {
        words.map(Limb::new)
    }

    #[test]
    fn halving_brings_the_top_bit_down_only_where_the_mask_says() {
        let top = 1 << (Word::BITS - 1);
        let mut halved = limbs([3, 1]);
        halve_masked(&mut halved, 1, Word::MAX);
        assert_eq!(halved, limbs([1 | top, top]));
        let mut kept = limbs([3, 1]);
        halve_masked(&mut kept, 1, 0);
        assert_eq!(kept, limbs([3, 1]));
    }

    #[test]
    fn selecting_takes_the_source_only_where_the_mask_says() {
        let mut value = limbs([1, 2]);
        select_assign(&mut value, &limbs([3, 4]), 0);
        assert_eq!(value, limbs([1, 2]));
        select_assign(&mut value, &limbs([3, 4]), Word::MAX);
        assert_eq!(value, limbs([3, 4]));
    }

    #[test]
    fn only_one_is_one() {
        assert_eq!(is_one(&limbs([1, 0])), 1);
        for words in [[0, 0], [1, 1], [3, 0], [0, 1], [Word::MAX, Word::MAX]] {
            assert_eq!(is_one(&limbs(words)), 0, "{words:?}");
        }
    }
}
