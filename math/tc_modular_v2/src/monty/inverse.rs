//! The word inverse that Montgomery reduction takes from the modulus.

use tc_bigint_v2::Word;

/// `-word⁻¹ mod 2^Word::BITS` for an odd `word`, by Newton's iteration:
/// `1` has the lowest bit right, and every step doubles the bits that are.
/// Constant time.
pub(crate) fn neg_inverse(word: Word) -> Word {
    let mut inverse: Word = 1;
    for _ in 0..Word::BITS.ilog2() {
        inverse = inverse.wrapping_mul(Word::wrapping_sub(2, word.wrapping_mul(inverse)));
    }
    inverse.wrapping_neg()
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::Word;

    use super::neg_inverse;

    #[test]
    fn the_inverse_times_the_word_is_minus_one() {
        for word in [1, 3, 5, 0xff, Word::MAX / 3, Word::MAX - 2, Word::MAX] {
            assert_eq!(word.wrapping_mul(neg_inverse(word)), Word::MAX, "{word}");
        }
    }
}
