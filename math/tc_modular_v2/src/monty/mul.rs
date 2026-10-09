//! Montgomery multiplication on limbs.

use tc_bigint_v2::{Limb, WideWord, Word};

use super::add::reduce_once;

/// `lhs · rhs · R⁻¹ mod modulus` into `output`, for `R = 2^(n · Word::BITS)`
/// at the width `n` of the modulus, which all four share; `inverse` is
/// `-modulus⁻¹ mod 2^Word::BITS`, and the product of the operands must be
/// below `modulus · R`, as for two values below the modulus. By coarsely
/// integrated operand scanning: each word of `rhs` is multiplied in, and
/// the multiple of the modulus that clears the low word is added before
/// that word is shifted out, so the running value keeps the width with one
/// word over. `output` must not be either operand. Constant time.
pub(crate) fn monty_mul(
    output: &mut [Limb],
    lhs: &[Limb],
    rhs: &[Limb],
    modulus: &[Limb],
    inverse: Word,
) {
    output.fill(Limb::new(0));
    let mut top: Word = 0;
    for word in rhs {
        // output + lhs · word, with the carry into `top` and over it.
        let factor = WideWord::from(word.to_word());
        let mut carry: WideWord = 0;
        for (limb, other) in output.iter_mut().zip(lhs) {
            let sum =
                WideWord::from(limb.to_word()) + WideWord::from(other.to_word()) * factor + carry;
            *limb = Limb::new(sum as Word);
            carry = sum >> Word::BITS;
        }
        let sum = WideWord::from(top) + carry;
        top = sum as Word;
        let over = (sum >> Word::BITS) as Word;

        // + q · modulus, which clears the low word, then down one word.
        let q = WideWord::from(output[0].to_word().wrapping_mul(inverse));
        let mut carry = (WideWord::from(output[0].to_word())
            + q * WideWord::from(modulus[0].to_word()))
            >> Word::BITS;
        for (index, limb) in modulus.iter().enumerate().skip(1) {
            let sum = WideWord::from(output[index].to_word())
                + q * WideWord::from(limb.to_word())
                + carry;
            output[index - 1] = Limb::new(sum as Word);
            carry = sum >> Word::BITS;
        }
        let sum = WideWord::from(top) + carry;
        output[modulus.len() - 1] = Limb::new(sum as Word);
        top = over + (sum >> Word::BITS) as Word;
    }
    // Below twice the modulus by now.
    reduce_once(output, top, modulus);
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{Limb, WideWord, Word};

    use super::monty_mul;
    use crate::monty::neg_inverse;

    #[test]
    fn the_product_times_r_is_the_plain_product_modulo_one_word() {
        for m in [1, 3, 0xff, Word::MAX / 3, Word::MAX - 2, Word::MAX] {
            for a in [0, 1, 2, m / 2, m - 1] {
                for b in [0, 1, 3, m / 3, m - 1] {
                    let (a, b) = (a % m, b % m);
                    let mut output = [Limb::new(0)];
                    let operands = ([Limb::new(a)], [Limb::new(b)], [Limb::new(m)]);
                    monty_mul(
                        &mut output,
                        &operands.0,
                        &operands.1,
                        &operands.2,
                        neg_inverse(m),
                    );
                    let wide = |word: Word| WideWord::from(word);
                    let shifted = (wide(output[0].to_word()) << Word::BITS) % wide(m);
                    assert!(output[0].to_word() < m.max(1), "{a} {b} {m}");
                    assert_eq!(shifted, wide(a) * wide(b) % wide(m), "{a} {b} {m}");
                }
            }
        }
    }
}
