//! Full products of unsigned limbs for the heap types: schoolbook for short
//! operands, Karatsuba from [`KARATSUBA_THRESHOLD`] limbs on.
//!
//! The work follows the values, skipping zero limbs and splitting by the
//! trimmed lengths, so everything here is variable time: only for public
//! values.

use alloc::{vec, vec::Vec};

use super::{Limb, WideWord, Word, add_assign_limbs, sub_assign_limbs, trimmed_len_unsigned};

/// The operand length, in limbs, from which Karatsuba pays, carried over
/// from `tc_bigint`: on 64-bit targets 2048-bit operands, the size of RSA,
/// take it, while 256-bit ones, the size of elliptic curves, stay on the
/// schoolbook path.
const KARATSUBA_THRESHOLD: usize = 32;

/// The product of unsigned `lhs` and `rhs` in `lhs.len() + rhs.len()`
/// limbs, or no limbs when either is zero. Variable time.
pub(crate) fn mul_limbs(lhs: &[Limb], rhs: &[Limb]) -> Vec<Limb> {
    let (lhs_len, rhs_len) = (trimmed_len_unsigned(lhs), trimmed_len_unsigned(rhs));
    if lhs_len == 0 || rhs_len == 0 {
        return Vec::new();
    }
    let mut product = vec![Limb::new(0); lhs.len() + rhs.len()];
    mul_add_into(&lhs[..lhs_len], &rhs[..rhs_len], &mut product);
    product
}

/// Adds `lhs * rhs` into `out`, which holds the sum.
fn mul_add_into(lhs: &[Limb], rhs: &[Limb], out: &mut [Limb]) {
    let (short, long) = match lhs.len() <= rhs.len() {
        true => (lhs, rhs),
        false => (rhs, lhs),
    };
    if short.len() < KARATSUBA_THRESHOLD {
        schoolbook_add_into(short, long, out);
    } else if short.len() < long.len() / 2 {
        // too uneven to split evenly: one chunk of the longer one at a time
        for (index, chunk) in long.chunks(short.len()).enumerate() {
            mul_add_into(short, chunk, &mut out[index * short.len()..]);
        }
    } else {
        karatsuba_add_into(short, long, out);
    }
}

/// Adds `short * long` into `out` row by row.
fn schoolbook_add_into(short: &[Limb], long: &[Limb], out: &mut [Limb]) {
    for (i, x) in short.iter().enumerate() {
        let x = WideWord::from(x.to_word());
        if x == 0 {
            continue;
        }
        let mut carry: Word = 0;
        for (j, y) in long.iter().enumerate() {
            let t = x * WideWord::from(y.to_word())
                + WideWord::from(out[i + j].to_word())
                + WideWord::from(carry);
            out[i + j] = Limb::new(t as Word);
            carry = (t >> Word::BITS) as Word;
        }
        add_carry(&mut out[i + long.len()..], carry);
    }
}

/// Adds `short * long` into `out` from three half-size products: with the
/// operands split at `X` into `low + high X`, the product is
/// `low_product + middle X + high_product X^2`, and `middle`, the cross
/// terms, comes from one product of the sums instead of two.
fn karatsuba_add_into(short: &[Limb], long: &[Limb], out: &mut [Limb]) {
    let split = long.len() / 2;
    let (short_low, short_high) = short.split_at(split);
    let (long_low, long_high) = long.split_at(split);
    let low = mul_limbs(short_low, long_low);
    let high = mul_limbs(short_high, long_high);
    let sums = mul_limbs(&sum(short_low, short_high), &sum(long_low, long_high));

    // the middle term is the product of the sums less the other two, which
    // never goes below zero, so adding before subtracting never borrows
    add_assign_limbs(out, &low, 0);
    add_assign_limbs(&mut out[split..], &sums, 0);
    sub_assign_limbs(&mut out[split..], &low, 0);
    sub_assign_limbs(&mut out[split..], &high, 0);
    add_assign_limbs(&mut out[2 * split..], &high, 0);
}

/// `a + b` in one limb more than the longer of them.
fn sum(a: &[Limb], b: &[Limb]) -> Vec<Limb> {
    let (long, short) = match a.len() >= b.len() {
        true => (a, b),
        false => (b, a),
    };
    let mut sum = Vec::with_capacity(long.len() + 1);
    sum.extend_from_slice(long);
    let carry = add_assign_limbs(&mut sum, short, 0);
    sum.push(Limb::new(carry));
    sum
}

/// Adds `carry` into `limbs` from the bottom, stopping once nothing carries.
fn add_carry(limbs: &mut [Limb], mut carry: Word) {
    for limb in limbs {
        if carry == 0 {
            break;
        }
        let (sum, overflow) = limb.to_word().overflowing_add(carry);
        *limb = Limb::new(sum);
        carry = Word::from(overflow);
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::{KARATSUBA_THRESHOLD, mul_limbs, schoolbook_add_into};
    use crate::{Limb, Word};

    /// `len` limbs from a xorshift sequence, to fill operands without a
    /// random number generator.
    fn limbs(len: usize, seed: u64) -> Vec<Limb> {
        let mut state = seed;
        (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                Limb::new(state as Word)
            })
            .collect()
    }

    fn schoolbook(lhs: &[Limb], rhs: &[Limb]) -> Vec<Limb> {
        let mut product = vec![Limb::new(0); lhs.len() + rhs.len()];
        schoolbook_add_into(lhs, rhs, &mut product);
        product
    }

    #[test]
    fn karatsuba_matches_schoolbook_at_even_and_uneven_lengths() {
        let t = KARATSUBA_THRESHOLD;
        for (lhs_len, rhs_len) in [
            (t, t),
            (t + 1, t),
            (2 * t + 3, 2 * t),
            (t, 3 * t),
            (t + 5, 7 * t),
        ] {
            let (lhs, rhs) = (limbs(lhs_len, 1), limbs(rhs_len, 2));
            assert_eq!(
                mul_limbs(&lhs, &rhs),
                schoolbook(&lhs, &rhs),
                "{lhs_len} {rhs_len}"
            );
        }
    }

    #[test]
    fn karatsuba_carries_through_operands_of_all_ones() {
        let t = KARATSUBA_THRESHOLD;
        let ones = vec![Limb::new(Word::MAX); 3 * t];
        assert_eq!(mul_limbs(&ones, &ones), schoolbook(&ones, &ones));
        assert_eq!(mul_limbs(&ones[..t], &ones), schoolbook(&ones[..t], &ones));
    }

    #[test]
    fn the_product_has_both_lengths_together_even_after_trimming() {
        let mut lhs = limbs(KARATSUBA_THRESHOLD, 3);
        lhs.extend([Limb::new(0); 3]);
        let rhs = limbs(KARATSUBA_THRESHOLD + 2, 4);
        let product = mul_limbs(&lhs, &rhs);
        assert_eq!(product.len(), lhs.len() + rhs.len());
        assert_eq!(product, schoolbook(&lhs, &rhs));
    }

    #[test]
    fn a_zero_operand_gives_no_limbs() {
        assert!(mul_limbs(&[Limb::new(0); 4], &limbs(5, 5)).is_empty());
        assert!(mul_limbs(&limbs(5, 5), &[]).is_empty());
    }
}
