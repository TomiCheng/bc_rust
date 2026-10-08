//! Reading integers from text, as the primitive integers' `from_str_radix`
//! does. The work follows the text, so everything here is variable time:
//! only for public values.

use crate::{Limb, ParseBigIntError, WideWord, Word};

/// The sign of `text` and its digits in `radix`: an optional `+` or `-`,
/// then at least one digit, in either case, and nothing else. Variable
/// time.
pub(crate) fn split(text: &str, radix: u32) -> Result<(bool, &[u8]), ParseBigIntError> {
    if !(2..=36).contains(&radix) {
        return Err(ParseBigIntError::InvalidRadix);
    }
    let (negative, digits) = match text.as_bytes() {
        [b'-', rest @ ..] => (true, rest),
        [b'+', rest @ ..] => (false, rest),
        all => (false, all),
    };
    if digits.is_empty()
        || !digits
            .iter()
            .all(|&digit| char::from(digit).is_digit(radix))
    {
        return Err(ParseBigIntError::InvalidDigit);
    }
    Ok((negative, digits))
}

/// Reads `digits`, which [`split`] checked, in `radix` into unsigned
/// `limbs`, which start at zero, and returns whether the value fitted.
/// Variable time.
pub(crate) fn accumulate(limbs: &mut [Limb], digits: &[u8], radix: u32) -> bool {
    for &digit in digits {
        // times the radix, plus the digit
        let mut carry = Word::from(char::from(digit).to_digit(radix).unwrap_or(0));
        for limb in limbs.iter_mut() {
            let t = WideWord::from(limb.to_word()) * WideWord::from(radix) + WideWord::from(carry);
            *limb = Limb::new(t as Word);
            carry = (t >> Word::BITS) as Word;
        }
        if carry != 0 {
            return false;
        }
    }
    true
}

/// Whether the magnitude in `limbs` fits their width as two's complement
/// with the given sign: below the top bit, or exactly the top bit when
/// negative, which is the most negative value. Variable time.
pub(crate) fn fits_signed(limbs: &[Limb], negative: bool) -> bool {
    let Some((top, rest)) = limbs.split_last() else {
        return true;
    };
    let sign_bit = 1 << (Word::BITS - 1);
    top.to_word() < sign_bit
        || (negative && top.to_word() == sign_bit && rest.iter().all(|limb| limb.to_word() == 0))
}

/// The limbs that `count` digits in `radix` can need, plus `extra` bits:
/// each digit counts as the bits that the largest digit takes, so the
/// width follows the length of the text, not the value. Constant time.
#[cfg(feature = "alloc")]
pub(crate) fn limbs_for_digits(count: usize, radix: u32, extra: usize) -> usize {
    let bits_per_digit = (u32::BITS - (radix - 1).leading_zeros()) as usize;
    (count * bits_per_digit + extra).div_ceil(Word::BITS as usize)
}
