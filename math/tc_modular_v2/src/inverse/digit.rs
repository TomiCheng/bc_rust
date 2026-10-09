//! The signed digit of safegcd, sized to the word: 62 bits in `i64` on
//! 64-bit targets and 30 bits in `i32` otherwise, two bits short of the
//! word, with products in the type twice as wide.

use tc_bigint_v2::Word;

#[cfg(target_pointer_width = "64")]
/// One signed digit, of [`DIGIT_BITS`] bits but the top one, which keeps
/// the sign and whatever lies above.
pub(crate) type Digit = i64;
#[cfg(not(target_pointer_width = "64"))]
/// One signed digit, of [`DIGIT_BITS`] bits but the top one, which keeps
/// the sign and whatever lies above.
pub(crate) type Digit = i32;

#[cfg(target_pointer_width = "64")]
/// Twice the width of [`Digit`], for products and carries.
pub(super) type WideDigit = i128;
#[cfg(not(target_pointer_width = "64"))]
/// Twice the width of [`Digit`], for products and carries.
pub(super) type WideDigit = i64;

/// The bits a digit holds below the top one.
pub(super) const DIGIT_BITS: u32 = Word::BITS - 2;

// A digit is as wide as the word, so that one converts to the other.
const _: () = assert!(Digit::BITS == Word::BITS);
