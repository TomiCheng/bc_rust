//! Writing integers as text without allocating, honouring the formatter's
//! sign, `#`, width, fill, alignment and zero-padding flags as the
//! primitive integers do. The digits show the value, so everything here is
//! variable time: only for public values.

use core::fmt::{self, Write};

use crate::{Limb, WideWord, Word};

/// The digits of the largest power of ten that a word holds.
const CHUNK_DIGITS: u32 = Word::MAX.ilog10();

/// The largest power of ten that a word holds.
const CHUNK: Word = (10 as Word).pow(CHUNK_DIGITS);

/// Divides unsigned `limbs` down to zero by the largest power of ten that a
/// word holds, writes each remainder into `chunks`, least significant
/// first, and returns how many there are: none for zero. `chunks` needs
/// two words for each limb. Variable time.
pub(crate) fn decimal_chunks(limbs: &mut [Limb], chunks: &mut [Word]) -> usize {
    let mut len = significant_len(limbs);
    let mut count = 0;
    while len > 0 {
        let mut remainder: WideWord = 0;
        for limb in limbs[..len].iter_mut().rev() {
            let numerator = remainder << Word::BITS | WideWord::from(limb.to_word());
            *limb = Limb::new((numerator / WideWord::from(CHUNK)) as Word);
            remainder = numerator % WideWord::from(CHUNK);
        }
        chunks[count] = remainder as Word;
        count += 1;
        len = significant_len(&limbs[..len]);
    }
    count
}

/// Writes the decimal number that `chunks` holds, least significant first,
/// after a minus sign when `negative`, padded as `f` asks. Variable time.
pub(crate) fn write_decimal(
    f: &mut fmt::Formatter<'_>,
    negative: bool,
    chunks: &[Word],
) -> fmt::Result {
    let Some((&top, rest)) = chunks.split_last() else {
        return pad_integral(f, false, "", 1, |f| f.write_char('0'));
    };
    let count = (top.ilog10() + 1) as usize + rest.len() * CHUNK_DIGITS as usize;
    pad_integral(f, negative, "", count, |f| {
        write!(f, "{top}")?;
        for chunk in rest.iter().rev() {
            write!(f, "{chunk:0width$}", width = CHUNK_DIGITS as usize)?;
        }
        Ok(())
    })
}

/// Writes unsigned `limbs` in the radix whose digits take `bits` bits,
/// after a minus sign when `negative` and, under `#`, `prefix`, padded as
/// `f` asks; `upper` picks capital letters. Variable time.
pub(crate) fn write_digits(
    f: &mut fmt::Formatter<'_>,
    negative: bool,
    limbs: &[Limb],
    bits: u32,
    prefix: &str,
    upper: bool,
) -> fmt::Result {
    let len = significant_len(limbs);
    let significant_bits = match len {
        0 => 0,
        _ => len * Word::BITS as usize - limbs[len - 1].to_word().leading_zeros() as usize,
    };
    let count = significant_bits.div_ceil(bits as usize).max(1);
    let letters = match upper {
        true => b"0123456789ABCDEF",
        false => b"0123456789abcdef",
    };
    pad_integral(f, negative, prefix, count, |f| {
        for digit in (0..count).rev() {
            let value = bits_at(limbs, digit * bits as usize, bits);
            f.write_char(char::from(letters[value as usize]))?;
        }
        Ok(())
    })
}

/// The `count` bits of `limbs` from bit `start` up, zero past their end.
fn bits_at(limbs: &[Limb], start: usize, count: u32) -> Word {
    let (index, shift) = (start / Word::BITS as usize, start as u32 % Word::BITS);
    let word = |index: usize| limbs.get(index).map_or(0, |limb| limb.to_word());
    let mut bits = word(index) >> shift;
    if shift != 0 {
        bits |= word(index + 1) << (Word::BITS - shift);
    }
    bits & ((1 << count) - 1)
}

/// The length of `limbs` without their leading zero limbs.
fn significant_len(limbs: &[Limb]) -> usize {
    limbs
        .iter()
        .rposition(|limb| limb.to_word() != 0)
        .map_or(0, |top| top + 1)
}

/// Writes the sign, `prefix` under `#`, and the `count` digits that
/// `digits` writes, padded to the width `f` asks for, as
/// [`fmt::Formatter::pad_integral`] does for digits it already has.
fn pad_integral(
    f: &mut fmt::Formatter<'_>,
    negative: bool,
    prefix: &str,
    count: usize,
    digits: impl FnOnce(&mut fmt::Formatter<'_>) -> fmt::Result,
) -> fmt::Result {
    let sign = match (negative, f.sign_plus()) {
        (true, _) => "-",
        (false, true) => "+",
        (false, false) => "",
    };
    let prefix = if f.alternate() { prefix } else { "" };
    let len = sign.len() + prefix.len() + count;
    let padding = f.width().map_or(0, |width| width.saturating_sub(len));
    if f.sign_aware_zero_pad() {
        // zeros go between the sign and prefix and the digits
        f.write_str(sign)?;
        f.write_str(prefix)?;
        for _ in 0..padding {
            f.write_char('0')?;
        }
        return digits(f);
    }
    let (before, after) = match f.align() {
        Some(fmt::Alignment::Left) => (0, padding),
        Some(fmt::Alignment::Center) => (padding / 2, padding - padding / 2),
        Some(fmt::Alignment::Right) | None => (padding, 0),
    };
    let fill = f.fill();
    for _ in 0..before {
        f.write_char(fill)?;
    }
    f.write_str(sign)?;
    f.write_str(prefix)?;
    digits(f)?;
    for _ in 0..after {
        f.write_char(fill)?;
    }
    Ok(())
}
