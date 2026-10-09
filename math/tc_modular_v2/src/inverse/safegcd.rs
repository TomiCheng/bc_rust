//! The modular inverse for an odd modulus by safegcd, in constant time.

use tc_bigint_v2::{Limb, WideWord, Word};

use super::digit::{DIGIT_BITS, Digit, WideDigit};
use crate::monty::neg_inverse;

/// The low [`DIGIT_BITS`] bits of a digit.
const MASK: Digit = (1 << DIGIT_BITS) - 1;

/// The shift that takes a digit to its sign, all ones or all zeros.
const SIGN: u32 = Digit::BITS - 1;

/// The rows of a limb each that the scratch of [`safegcd_inverse`] takes:
/// five working values, each in at most twice as many digits as the width
/// has limbs.
pub(crate) const SCRATCH_ROWS: usize = 10;

/// `value⁻¹ mod modulus` for an odd `modulus`, with `value` below it, both
/// of the width of `zero`, which gives the storage of the result; `None`
/// when there is no inverse. `scratch` holds the working digits, at least
/// [`SCRATCH_ROWS`] times as many as the width has limbs.
///
/// Bernstein and Yang's safegcd, in its half-delta form, as Bouncy Castle
/// has it: `f` and `g` start at the modulus and the value, and batches of
/// divsteps on their low digits give a transition matrix, applied to them
/// and to the coefficients `d` and `e` at once. The number of batches comes
/// from the bits of the width alone, enough for any value; at the end `f`
/// is `±1` and `g` zero exactly when there is an inverse, which `d` is up
/// to the sign of `f`.
///
/// Constant time, apart from whether there is an inverse, which the
/// `Option` shows.
pub(crate) fn safegcd_inverse<S>(
    value: &[Limb],
    modulus: &[Limb],
    zero: &S,
    scratch: &mut [Digit],
) -> Option<S>
where
    S: AsMut<[Limb]> + Clone,
{
    let bits = modulus.len() as u32 * Word::BITS;
    let len = bits.div_ceil(DIGIT_BITS) as usize;
    scratch.fill(0);
    let (d, rest) = scratch.split_at_mut(len);
    let (e, rest) = rest.split_at_mut(len);
    let (f, rest) = rest.split_at_mut(len);
    let (g, rest) = rest.split_at_mut(len);
    let m = &mut rest[..len];

    e[0] = 1;
    encode(value, g);
    encode(modulus, m);
    f.copy_from_slice(m);
    // `modulus⁻¹ mod 2^Word::BITS`, of which the low digit is what counts.
    let m0_inverse = neg_inverse(modulus[0].to_word()).wrapping_neg() as Digit;

    let mut theta = 0;
    let mut transition = [0; 4];
    for _ in 0..max_divsteps(bits).div_ceil(DIGIT_BITS) {
        theta = divsteps(theta, f[0], g[0], &mut transition);
        update_de(d, e, &transition, m0_inverse, m);
        update_fg(f, g, &transition);
    }

    let sign_f = f[len - 1] >> SIGN;
    conditional_negate(sign_f, f);
    conditional_normalize(sign_f, d, m);
    let invertible = equal_to(f, 1) & equal_to(g, 0);
    let mut inverse = zero.clone();
    decode(d, inverse.as_mut());
    (invertible == 1).then_some(inverse)
}

/// The half-delta divsteps that an input of `bits` bits needs at most, as
/// Bouncy Castle bounds them.
fn max_divsteps(bits: u32) -> u32 {
    ((150_964 * u64::from(bits) + 99_243) >> 16) as u32
}

/// [`DIGIT_BITS`] half-delta divsteps on the low digits `f0` and `g0`, from
/// `theta`. The transition matrix, scaled by `2^DIGIT_BITS`, goes into
/// `transition`, and the new `theta` comes back. Constant time.
fn divsteps(mut theta: Digit, f0: Digit, g0: Digit, transition: &mut [Digit; 4]) -> Digit {
    let (mut u, mut v, mut q, mut r): (Digit, Digit, Digit, Digit) =
        (1 << DIGIT_BITS, 0, 0, 1 << DIGIT_BITS);
    let (mut f, mut g) = (f0, g0);
    for _ in 0..DIGIT_BITS {
        let c1 = theta >> SIGN;
        let c2 = -(g & 1);
        let (x, y, z) = (f ^ c1, u ^ c1, v ^ c1);
        g = g.wrapping_sub(x & c2);
        q = q.wrapping_sub(y & c2);
        r = r.wrapping_sub(z & c2);
        let c3 = c2 & !c1;
        theta = (theta ^ c3).wrapping_add(1);
        f = f.wrapping_add(g & c3);
        u = u.wrapping_add(q & c3);
        v = v.wrapping_add(r & c3);
        g >>= 1;
        q >>= 1;
        r >>= 1;
    }
    *transition = [u, v, q, r];
    theta
}

/// Applies the transition matrix to the coefficients `d` and `e`, adding
/// the multiples of the modulus that keep them whole after the shift.
/// Constant time.
fn update_de(
    d: &mut [Digit],
    e: &mut [Digit],
    transition: &[Digit; 4],
    m0_inverse: Digit,
    modulus: &[Digit],
) {
    let [u, v, q, r] = *transition;
    let wide = WideDigit::from;
    let last = d.len() - 1;
    let (sign_d, sign_e) = (d[last] >> SIGN, e[last] >> SIGN);
    let mut md = (u & sign_d).wrapping_add(v & sign_e);
    let mut me = (q & sign_d).wrapping_add(r & sign_e);
    let mut carry_d = wide(u) * wide(d[0]) + wide(v) * wide(e[0]);
    let mut carry_e = wide(q) * wide(d[0]) + wide(r) * wide(e[0]);
    md = md.wrapping_sub(m0_inverse.wrapping_mul(carry_d as Digit).wrapping_add(md) & MASK);
    me = me.wrapping_sub(m0_inverse.wrapping_mul(carry_e as Digit).wrapping_add(me) & MASK);
    carry_d += wide(modulus[0]) * wide(md);
    carry_e += wide(modulus[0]) * wide(me);
    carry_d >>= DIGIT_BITS;
    carry_e >>= DIGIT_BITS;
    for (index, &digit) in modulus.iter().enumerate().skip(1) {
        carry_d += wide(u) * wide(d[index]) + wide(v) * wide(e[index]) + wide(digit) * wide(md);
        carry_e += wide(q) * wide(d[index]) + wide(r) * wide(e[index]) + wide(digit) * wide(me);
        d[index - 1] = carry_d as Digit & MASK;
        e[index - 1] = carry_e as Digit & MASK;
        carry_d >>= DIGIT_BITS;
        carry_e >>= DIGIT_BITS;
    }
    d[last] = carry_d as Digit;
    e[last] = carry_e as Digit;
}

/// Applies the transition matrix to `f` and `g`, whose low digits it
/// clears, and shifts them down a digit. Constant time.
fn update_fg(f: &mut [Digit], g: &mut [Digit], transition: &[Digit; 4]) {
    let [u, v, q, r] = *transition;
    let wide = WideDigit::from;
    let mut carry_f = wide(u) * wide(f[0]) + wide(v) * wide(g[0]);
    let mut carry_g = wide(q) * wide(f[0]) + wide(r) * wide(g[0]);
    carry_f >>= DIGIT_BITS;
    carry_g >>= DIGIT_BITS;
    for index in 1..f.len() {
        carry_f += wide(u) * wide(f[index]) + wide(v) * wide(g[index]);
        carry_g += wide(q) * wide(f[index]) + wide(r) * wide(g[index]);
        f[index - 1] = carry_f as Digit & MASK;
        g[index - 1] = carry_g as Digit & MASK;
        carry_f >>= DIGIT_BITS;
        carry_g >>= DIGIT_BITS;
    }
    let last = f.len() - 1;
    f[last] = carry_f as Digit;
    g[last] = carry_g as Digit;
}

/// Negates `value` where `condition` is all ones, and leaves it where it is
/// zero. Constant time.
fn conditional_negate(condition: Digit, value: &mut [Digit]) {
    let last = value.len() - 1;
    let mut carry: WideDigit = 0;
    for digit in &mut value[..last] {
        carry += WideDigit::from((*digit ^ condition).wrapping_sub(condition));
        *digit = carry as Digit & MASK;
        carry >>= DIGIT_BITS;
    }
    carry += WideDigit::from((value[last] ^ condition).wrapping_sub(condition));
    value[last] = carry as Digit;
}

/// Brings `value`, between minus twice the modulus and the modulus, into
/// `[0, modulus)`, negated first where `condition_negate` is all ones: the
/// modulus is added where the value is negative, before the negation and
/// after it. Constant time.
fn conditional_normalize(condition_negate: Digit, value: &mut [Digit], modulus: &[Digit]) {
    let last = value.len() - 1;
    let mut carry: WideDigit = 0;
    let condition_add = value[last] >> SIGN;
    for index in 0..last {
        let digit = value[index].wrapping_add(modulus[index] & condition_add);
        let digit = (digit ^ condition_negate).wrapping_sub(condition_negate);
        carry += WideDigit::from(digit);
        value[index] = carry as Digit & MASK;
        carry >>= DIGIT_BITS;
    }
    let digit = value[last].wrapping_add(modulus[last] & condition_add);
    let digit = (digit ^ condition_negate).wrapping_sub(condition_negate);
    carry += WideDigit::from(digit);
    value[last] = carry as Digit;

    let mut carry: WideDigit = 0;
    let condition_add = value[last] >> SIGN;
    for index in 0..last {
        carry += WideDigit::from(value[index].wrapping_add(modulus[index] & condition_add));
        value[index] = carry as Digit & MASK;
        carry >>= DIGIT_BITS;
    }
    carry += WideDigit::from(value[last].wrapping_add(modulus[last] & condition_add));
    value[last] = carry as Digit;
}

/// One when `value` is `expected`, a single digit, and zero otherwise.
/// Constant time.
fn equal_to(value: &[Digit], expected: Digit) -> Digit {
    let mut difference = value[0] ^ expected;
    for digit in &value[1..] {
        difference |= *digit;
    }
    let bits = difference as Word;
    ((bits | bits.wrapping_neg()) >> (Word::BITS - 1)) as Digit ^ 1
}

/// The limbs of `value` as digits of [`DIGIT_BITS`] bits, low first, into
/// `output`, which is long enough for them. Constant time.
fn encode(value: &[Limb], output: &mut [Digit]) {
    let mut limbs = value.iter();
    let (mut data, mut available): (WideWord, u32) = (0, 0);
    for digit in output.iter_mut() {
        if available < DIGIT_BITS {
            if let Some(limb) = limbs.next() {
                data |= WideWord::from(limb.to_word()) << available;
                available += Word::BITS;
            }
        }
        *digit = data as Digit & MASK;
        data >>= DIGIT_BITS;
        available = available.saturating_sub(DIGIT_BITS);
    }
}

/// The digits of `value`, none negative, back as limbs, low first, into
/// `output`. Constant time.
fn decode(value: &[Digit], output: &mut [Limb]) {
    let mut digits = value.iter();
    let (mut data, mut available): (WideWord, u32) = (0, 0);
    for limb in output.iter_mut() {
        while available < Word::BITS {
            let Some(&digit) = digits.next() else { break };
            data |= WideWord::from(digit as Word) << available;
            available += DIGIT_BITS;
        }
        *limb = Limb::new(data as Word);
        data >>= Word::BITS;
        available = available.saturating_sub(Word::BITS);
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{Limb, Word};

    use super::super::binary_inverse;
    use super::{SCRATCH_ROWS, safegcd_inverse};
    use crate::inverse::Digit;

    /// The next word of a xorshift generator.
    fn next(state: &mut u64) -> Word {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state as Word
    }

    fn safegcd<const N: usize>(value: &[Limb; N], modulus: &[Limb; N]) -> Option<[Limb; N]> {
        let mut scratch = [[0 as Digit; N]; SCRATCH_ROWS];
        safegcd_inverse(
            value,
            modulus,
            &[Limb::new(0); N],
            scratch.as_flattened_mut(),
        )
    }

    /// Odd moduli of `N` limbs, every other one full and the rest with high
    /// limbs left empty, and values below them, from the low end, the high
    /// end and in between.
    fn agrees_with_the_binary_inverse<const N: usize>(seed: u64, rounds: usize) {
        let mut state = seed;
        for round in 0..rounds {
            let mut modulus = [Limb::new(0); N];
            let used = if round % 2 == 0 { N } else { 1 + round % N };
            for limb in &mut modulus[..used] {
                *limb = Limb::new(next(&mut state));
            }
            modulus[0] = Limb::new(modulus[0].to_word() | 1);
            let below = |value: [Limb; N]| -> [Limb; N] {
                // The value with its top used limb cut under the modulus.
                let mut value = value;
                value[used..].fill(Limb::new(0));
                let top = modulus[used - 1].to_word();
                value[used - 1] = Limb::new(if top == 0 {
                    0
                } else {
                    value[used - 1].to_word() % top
                });
                value
            };
            let random = below([(); N].map(|()| Limb::new(next(&mut state))));
            let mut minus_one = modulus;
            minus_one[0] = Limb::new(modulus[0].to_word() - 1);
            let mut one = [Limb::new(0); N];
            one[0] = Limb::new(1);
            for value in [[Limb::new(0); N], one, minus_one, random] {
                let expected = binary_inverse(&value, &modulus, &[Limb::new(0); N]);
                assert_eq!(safegcd(&value, &modulus), expected, "{round}");
            }
        }
    }

    #[test]
    fn the_inverse_agrees_with_the_binary_inverse_at_every_width() {
        agrees_with_the_binary_inverse::<1>(1, 24);
        agrees_with_the_binary_inverse::<2>(2, 24);
        agrees_with_the_binary_inverse::<3>(3, 24);
        agrees_with_the_binary_inverse::<4>(4, 24);
        agrees_with_the_binary_inverse::<8>(8, 12);
        // Few rounds, as the binary inverse is slow at 2048 bits unoptimized.
        agrees_with_the_binary_inverse::<{ 2048 / Word::BITS as usize }>(32, 4);
    }

    #[test]
    fn known_inverses_and_their_absence_come_out() {
        let limbs = |word: Word| [Limb::new(word)];
        assert_eq!(safegcd(&limbs(3), &limbs(7)), Some(limbs(5)));
        assert_eq!(safegcd(&limbs(0), &limbs(1)), Some(limbs(0)));
        assert_eq!(safegcd(&limbs(6), &limbs(15)), None);
        assert_eq!(safegcd(&limbs(0), &limbs(15)), None);
        assert_eq!(
            safegcd(&limbs(Word::MAX - 1), &limbs(Word::MAX)),
            Some(limbs(Word::MAX - 1))
        );
    }
}
