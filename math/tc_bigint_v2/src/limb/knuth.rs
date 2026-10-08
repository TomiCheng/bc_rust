//! Long division of unsigned limbs for the heap types, by Knuth's
//! Algorithm D (The Art of Computer Programming, volume 2, 4.3.1), ported
//! from `tc_bigint`.
//!
//! The work follows the values, from the trimmed lengths to the quotient
//! estimates, so everything here is variable time: only for public values.

use alloc::{vec, vec::Vec};

use super::{
    Limb, WideWord, Word, add_assign_limbs, shl_assign_limbs, shr_assign_limbs,
    trimmed_len_unsigned,
};

/// The quotient and remainder of unsigned `dividend` by `divisor`, which
/// must not be zero, neither of them trimmed. The remainder is built in
/// the storage of `dividend`. Variable time.
pub(crate) fn knuth_div_rem(mut dividend: Vec<Limb>, divisor: &[Limb]) -> (Vec<Limb>, Vec<Limb>) {
    dividend.truncate(trimmed_len_unsigned(&dividend));
    let divisor = &divisor[..trimmed_len_unsigned(divisor)];
    debug_assert!(!divisor.is_empty());
    if dividend.len() < divisor.len() {
        return (Vec::new(), dividend);
    }
    if let [divisor] = divisor {
        // one limb: a word at a time, the quotient in place of the dividend
        let divisor = WideWord::from(divisor.to_word());
        let mut remainder: WideWord = 0;
        for limb in dividend.iter_mut().rev() {
            let numerator = remainder << Word::BITS | WideWord::from(limb.to_word());
            *limb = Limb::new((numerator / divisor) as Word);
            remainder = numerator % divisor;
        }
        return (dividend, vec![Limb::new(remainder as Word)]);
    }

    // Shifting both so that the top bit of the divisor is set keeps each
    // estimate of a quotient limb at most two too large; the dividend takes
    // one more limb for the bits shifted out.
    let shift = divisor[divisor.len() - 1].to_word().leading_zeros();
    let mut normalized = divisor.to_vec();
    shl_assign_limbs(&mut normalized, shift);
    dividend.push(Limb::new(0));
    shl_assign_limbs(&mut dividend, shift);

    let n = normalized.len();
    let mut quotient = vec![Limb::new(0); dividend.len() - n];
    let base = 1 << Word::BITS;
    let top = WideWord::from(normalized[n - 1].to_word());
    let next = WideWord::from(normalized[n - 2].to_word());
    for j in (0..quotient.len()).rev() {
        // estimate the quotient limb from the top two limbs, then correct it
        // by the next one
        let word = |index: usize| WideWord::from(dividend[index].to_word());
        let numerator = word(j + n) << Word::BITS | word(j + n - 1);
        let mut estimate = numerator / top;
        let mut rest = numerator % top;
        while estimate == base
            || (rest < base && estimate * next > (rest << Word::BITS) + word(j + n - 2))
        {
            estimate -= 1;
            rest += top;
        }

        // take estimate * divisor out of the dividend at j
        let mut borrow: WideWord = 0;
        for i in 0..n {
            let product = estimate * WideWord::from(normalized[i].to_word()) + borrow;
            let (difference, under) = dividend[j + i].to_word().overflowing_sub(product as Word);
            dividend[j + i] = Limb::new(difference);
            borrow = (product >> Word::BITS) + WideWord::from(under);
        }
        let (difference, negative) = dividend[j + n].to_word().overflowing_sub(borrow as Word);
        dividend[j + n] = Limb::new(difference);
        if negative {
            // the estimate was still one too large: add the divisor back
            estimate -= 1;
            let carry = add_assign_limbs(&mut dividend[j..j + n], &normalized, 0);
            dividend[j + n] = Limb::new(dividend[j + n].to_word().wrapping_add(carry));
        }
        quotient[j] = Limb::new(estimate as Word);
    }

    // the remainder is what is left of the dividend, shifted back
    dividend.truncate(n);
    shr_assign_limbs(&mut dividend, shift, 0);
    (quotient, dividend)
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::knuth_div_rem;
    use crate::limb::{add_assign_limbs, mul_limbs, trimmed_len_unsigned};
    use crate::{Limb, Word};

    /// Checks `dividend = quotient * divisor + remainder` with the
    /// remainder below the divisor.
    fn check(dividend: &[Limb], divisor: &[Limb]) {
        let (quotient, remainder) = knuth_div_rem(dividend.to_vec(), divisor);
        let mut rebuilt = mul_limbs(&quotient, divisor);
        rebuilt.resize(rebuilt.len().max(dividend.len()) + 1, Limb::new(0));
        assert_eq!(add_assign_limbs(&mut rebuilt, &remainder, 0), 0);
        let trimmed = |limbs: &[Limb]| limbs[..trimmed_len_unsigned(limbs)].to_vec();
        assert_eq!(
            trimmed(&rebuilt),
            trimmed(dividend),
            "{dividend:?} {divisor:?}"
        );
        let (remainder, divisor) = (trimmed(&remainder), trimmed(divisor));
        let below = remainder.len() < divisor.len()
            || (remainder.len() == divisor.len()
                && remainder
                    .iter()
                    .rev()
                    .map(|limb| limb.to_word())
                    .lt(divisor.iter().rev().map(|limb| limb.to_word())));
        assert!(below, "{dividend:?} {divisor:?}");
    }

    /// Words at the edges of the estimate and its corrections.
    const EDGES: [Word; 7] = [
        0,
        1,
        Word::MAX,
        Word::MAX - 1,
        1 << (Word::BITS - 1),
        (1 << (Word::BITS - 1)) - 1,
        (1 << (Word::BITS - 1)) + 1,
    ];

    #[test]
    fn every_three_limb_dividend_of_edge_words_divides_by_every_two_limb_divisor() {
        for a in EDGES {
            for b in EDGES {
                for c in EDGES {
                    let dividend = [Limb::new(a), Limb::new(b), Limb::new(c)];
                    for d in EDGES {
                        for e in EDGES.into_iter().filter(|&e| e != 0) {
                            check(&dividend, &[Limb::new(d), Limb::new(e)]);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn an_estimate_still_one_too_large_is_corrected_by_adding_the_divisor_back() {
        // from the tests of Hacker's Delight, at any limb width: the top
        // limbs estimate a quotient of 4 where it is 3
        let top = 1 << (Word::BITS - 1);
        let dividend = [Limb::new(3), Limb::new(0), Limb::new(top)];
        let divisor = [Limb::new(1), Limb::new(0), Limb::new(top >> 2)];
        let (quotient, _) = knuth_div_rem(dividend.to_vec(), &divisor);
        assert_eq!(quotient, [Limb::new(3)]);
        check(&dividend, &divisor);
    }

    #[test]
    fn long_operands_divide_back_to_the_dividend() {
        let mut state: u64 = 1;
        let mut limbs = |len: usize| -> Vec<Limb> {
            (0..len)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    Limb::new(state as Word)
                })
                .collect()
        };
        for (dividend_len, divisor_len) in [(40, 1), (40, 2), (40, 17), (70, 40), (17, 40)] {
            let (dividend, divisor) = (limbs(dividend_len), limbs(divisor_len));
            check(&dividend, &divisor);
        }
    }
}
