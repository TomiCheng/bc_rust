//! Arithmetic of limbs with a single word, which needs no storage of its
//! own.

use super::{Limb, WideWord, Word};

/// Adds `word` into `a` in place, from the bottom, and returns what is
/// carried out of the top limb: zero or one, or the word itself when `a`
/// has no limbs. Constant time: the length only decides how far the loop
/// runs.
pub(crate) fn add_assign_word(a: &mut [Limb], word: Word) -> Word {
    let mut carry = word;
    for limb in a.iter_mut() {
        let (sum, over) = limb.to_word().overflowing_add(carry);
        *limb = Limb::new(sum);
        carry = Word::from(over);
    }
    carry
}

/// Takes `word` from `a` in place, from the bottom, and returns what is
/// borrowed past the top limb: zero or one, or the word itself when `a`
/// has no limbs. Constant time: the length only decides how far the loop
/// runs.
pub(crate) fn sub_assign_word(a: &mut [Limb], word: Word) -> Word {
    let mut borrow = word;
    for limb in a.iter_mut() {
        let (difference, under) = limb.to_word().overflowing_sub(borrow);
        *limb = Limb::new(difference);
        borrow = Word::from(under);
    }
    borrow
}

/// Multiplies `a` by `word` in place, in one pass from the bottom, and
/// returns the word carried out of the top limb. Constant time: the length
/// only decides how far the loop runs.
pub(crate) fn mul_assign_word(a: &mut [Limb], word: Word) -> Word {
    let factor = WideWord::from(word);
    let mut carry: Word = 0;
    for limb in a.iter_mut() {
        let product = WideWord::from(limb.to_word()) * factor + WideWord::from(carry);
        *limb = Limb::new(product as Word);
        carry = (product >> Word::BITS) as Word;
    }
    carry
}

/// Divides `a` by `divisor`, which must not be zero, in place, and returns
/// the remainder, by long division a bit at a time from the top: the
/// remainder takes each bit and gives up the divisor whenever it can,
/// through a mask, so no hardware division sees the value. Constant time:
/// the length only decides how far the loop runs.
pub(crate) fn div_assign_word(a: &mut [Limb], divisor: Word) -> Word {
    let divisor = WideWord::from(divisor);
    let mut remainder: WideWord = 0;
    for limb in a.iter_mut().rev() {
        let word = limb.to_word();
        let mut quotient: Word = 0;
        for bit in (0..Word::BITS).rev() {
            remainder = remainder << 1 | WideWord::from((word >> bit) & 1);
            let (difference, borrow) = remainder.overflowing_sub(divisor);
            // all ones when the divisor went in
            let fits = WideWord::from(borrow).wrapping_sub(1);
            remainder = (difference & fits) | (remainder & !fits);
            quotient |= ((fits & 1) as Word) << bit;
        }
        *limb = Limb::new(quotient);
    }
    remainder as Word
}

/// Divides `a` by `divisor`, which must not be zero, in place, a limb at a
/// time from the top, and returns the remainder. Variable time, as the
/// hardware division takes longer for some operands: only for public
/// values.
#[cfg(feature = "alloc")]
pub(crate) fn div_assign_word_vartime(a: &mut [Limb], divisor: Word) -> Word {
    let divisor = WideWord::from(divisor);
    let mut remainder: WideWord = 0;
    for limb in a.iter_mut().rev() {
        let numerator = remainder << Word::BITS | WideWord::from(limb.to_word());
        *limb = Limb::new((numerator / divisor) as Word);
        remainder = numerator % divisor;
    }
    remainder as Word
}

#[cfg(test)]
mod tests {
    use super::{Limb, Word, add_assign_word, div_assign_word, mul_assign_word, sub_assign_word};

    #[test]
    fn the_carry_out_of_a_product_is_its_top_word() {
        let mut limbs = [Limb::new(Word::MAX), Limb::new(Word::MAX)];
        let carry = mul_assign_word(&mut limbs, Word::MAX);
        // (2^2w - 1)(2^w - 1) = 2^3w - 2^2w - 2^w + 1
        assert_eq!(limbs, [Limb::new(1), Limb::new(Word::MAX)]);
        assert_eq!(carry, Word::MAX - 1);
    }

    #[test]
    fn a_word_on_no_limbs_comes_back_whole() {
        assert_eq!(add_assign_word(&mut [], 7), 7);
        assert_eq!(sub_assign_word(&mut [], 7), 7);
        assert_eq!(div_assign_word(&mut [], 7), 0);
    }

    #[test]
    fn the_division_a_bit_at_a_time_matches_the_primitive_one() {
        for value in [0, 1, 255, u64::MAX, 1 << 63, u64::MAX / 3] {
            for divisor in [1, 2, 3, 10, 65_537, u32::MAX] {
                let mut limbs = [Limb::new(0); 2];
                for (index, limb) in limbs.iter_mut().enumerate() {
                    *limb = Limb::new((u128::from(value) >> (index as u32 * Word::BITS)) as Word);
                }
                let remainder = div_assign_word(&mut limbs, Word::from(divisor));
                let quotient = limbs.iter().enumerate().fold(0u128, |sum, (index, limb)| {
                    sum | u128::from(limb.to_word()) << (index as u32 * Word::BITS)
                });
                let (expected_quotient, expected_remainder) =
                    (value / u64::from(divisor), value % u64::from(divisor));
                assert_eq!(quotient, u128::from(expected_quotient), "{value} {divisor}");
                // the remainder is below the divisor, so it fits a word
                assert_eq!(remainder, expected_remainder as Word, "{value} {divisor}");
            }
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn both_divisions_agree() {
        use super::div_assign_word_vartime;

        for words in [
            [0, 0],
            [1, 0],
            [Word::MAX, Word::MAX],
            [12_345, 1 << (Word::BITS - 1)],
        ] {
            for divisor in [1, 3, 10, Word::from(u32::MAX)] {
                let (mut bitwise, mut hardware) = (words.map(Limb::new), words.map(Limb::new));
                let remainders = (
                    div_assign_word(&mut bitwise, divisor),
                    div_assign_word_vartime(&mut hardware, divisor),
                );
                assert_eq!(
                    (bitwise, remainders.0),
                    (hardware, remainders.1),
                    "{words:?} {divisor}"
                );
            }
        }
    }
}
