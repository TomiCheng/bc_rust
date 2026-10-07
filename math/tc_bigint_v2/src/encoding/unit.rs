//! Moving values between limbs and slices of narrower or wider units.
//!
//! Every helper takes its lengths from public widths and never branches on
//! the value, so all of them are constant time.

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

use crate::{ConversionError, Limb, Word};

const WORD_BITS: usize = Word::BITS as usize;

/// An encoding unit: `u8`, `u32` or `u64`.
pub(crate) trait Unit: Copy + Default {
    /// The width of the unit in bits.
    const BITS: u32;

    /// The low `BITS` bits of `bits`.
    fn from_bits(bits: u128) -> Self;

    /// The unit, zero-extended.
    fn to_bits(self) -> u128;
}

macro_rules! impl_unit {
    ($($ty:ty),*) => {$(
        impl Unit for $ty {
            const BITS: u32 = <$ty>::BITS;

            fn from_bits(bits: u128) -> Self {
                bits as $ty
            }

            fn to_bits(self) -> u128 {
                self as u128
            }
        }
    )*};
}

impl_unit!(u8, u32, u64);

/// The low `bits` bits set; `bits` is at most 64.
const fn mask(bits: usize) -> u128 {
    (1 << bits) - 1
}

/// All ones when the top bit of the last limb is set, zero otherwise and for
/// no limbs.
pub(crate) fn sign_fill(limbs: &[Limb]) -> Word {
    limbs.last().map_or(0, |top| {
        (0 as Word).wrapping_sub(top.to_word() >> (Word::BITS - 1))
    })
}

/// [`sign_fill`] of `len` two's-complement units read through `unit`.
pub(crate) fn input_fill<U: Unit>(len: usize, unit: impl Fn(usize) -> U) -> Word {
    match len {
        0 => 0,
        _ => (0 as Word).wrapping_sub((unit(len - 1).to_bits() >> (U::BITS - 1)) as Word),
    }
}

/// The units needed for `bits` bits.
pub(crate) fn units_for_bits<U: Unit>(bits: usize) -> usize {
    bits.div_ceil(U::BITS as usize)
}

/// The `index`-th unit of `limbs`, least significant first, reading `fill`
/// above them.
fn unit_at<U: Unit>(limbs: &[Limb], fill: Word, index: usize) -> U {
    let unit_bits = U::BITS as usize;
    let start = index * unit_bits;
    let mut value = 0;
    let mut taken = 0;
    while taken < unit_bits {
        let bit = start + taken;
        let shift = bit % WORD_BITS;
        let chunk = (WORD_BITS - shift).min(unit_bits - taken);
        let word = limbs
            .get(bit / WORD_BITS)
            .map_or(fill, |limb| limb.to_word());
        value |= ((word >> shift) as u128 & mask(chunk)) << taken;
        taken += chunk;
    }
    U::from_bits(value)
}

/// Fills `out` from `len` units read through `unit`, least significant
/// first, with `fill` above them.
pub(crate) fn decode_into<U: Unit>(
    len: usize,
    unit: impl Fn(usize) -> U,
    fill: Word,
    out: &mut [Limb],
) {
    let unit_bits = U::BITS as usize;
    for (index, limb) in out.iter_mut().enumerate() {
        let start = index * WORD_BITS;
        let mut word: Word = 0;
        let mut taken = 0;
        while taken < WORD_BITS {
            let bit = start + taken;
            let position = bit / unit_bits;
            let shift = bit % unit_bits;
            let chunk = (unit_bits - shift).min(WORD_BITS - taken);
            let source = match position < len {
                true => unit(position).to_bits() >> shift,
                false => fill as u128,
            };
            word |= ((source & mask(chunk)) as Word) << taken;
            taken += chunk;
        }
        *limb = Limb::new(word);
    }
}

/// [`decode_into`] limbs just wide enough for the `len` units.
#[cfg(feature = "alloc")]
pub(crate) fn decode_vec<U: Unit>(len: usize, unit: impl Fn(usize) -> U, fill: Word) -> Vec<Limb> {
    let mut limbs = vec![Limb::new(0); (len * U::BITS as usize).div_ceil(WORD_BITS)];
    decode_into(len, unit, fill, &mut limbs);
    limbs
}

/// Whether each of the `len` units equals the unit `limbs` hold at its
/// place, so that decoding them into `limbs` lost nothing.
pub(crate) fn fits<U: Unit>(
    limbs: &[Limb],
    fill: Word,
    len: usize,
    unit: impl Fn(usize) -> U,
) -> bool {
    let mut difference = 0;
    for index in 0..len {
        difference |= unit(index).to_bits() ^ unit_at::<U>(limbs, fill, index).to_bits();
    }
    difference == 0
}

/// Writes `length` units of `limbs` into the front of `output`, least
/// significant first.
pub(crate) fn write_le<U: Unit>(
    limbs: &[Limb],
    fill: Word,
    length: usize,
    output: &mut [U],
) -> Result<usize, ConversionError> {
    let output = output
        .get_mut(..length)
        .ok_or(ConversionError::BufferTooSmall)?;
    for (index, unit) in output.iter_mut().enumerate() {
        *unit = unit_at(limbs, fill, index);
    }
    Ok(length)
}

/// Writes `length` units of `limbs` into the front of `output`, most
/// significant first.
pub(crate) fn write_be<U: Unit>(
    limbs: &[Limb],
    fill: Word,
    length: usize,
    output: &mut [U],
) -> Result<usize, ConversionError> {
    let output = output
        .get_mut(..length)
        .ok_or(ConversionError::BufferTooSmall)?;
    for (index, unit) in output.iter_mut().rev().enumerate() {
        *unit = unit_at(limbs, fill, index);
    }
    Ok(length)
}
