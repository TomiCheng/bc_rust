//! The native word types and the limb that wraps them.

use core::fmt;

use tc_constant_time::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeOrd};
use tc_zeroize::Zeroize;

#[cfg(target_pointer_width = "64")]
/// The native word: `u64` on 64-bit targets, `u32` otherwise.
pub type Word = u64;
#[cfg(not(target_pointer_width = "64"))]
/// The native word: `u64` on 64-bit targets, `u32` otherwise.
pub type Word = u32;

#[cfg(target_pointer_width = "64")]
/// Twice the width of [`Word`], for carries and products.
pub type WideWord = u128;
#[cfg(not(target_pointer_width = "64"))]
/// Twice the width of [`Word`], for carries and products.
pub type WideWord = u64;

/// One storage word.
#[repr(transparent)]
#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct Limb(Word);

/// Prints `Limb(..)`, never the word, so a secret cannot reach a log or a
/// panic message. Constant time.
impl fmt::Debug for Limb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Limb").finish_non_exhaustive()
    }
}

impl Limb {
    /// Wraps a native word. Constant time.
    pub const fn new(word: Word) -> Self {
        Self(word)
    }

    /// Returns the native word. Constant time.
    pub const fn to_word(self) -> Word {
        self.0
    }
}

/// Constant time: delegates to [`Word`](Word).
impl ConditionallySelectable for Limb {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(ConditionallySelectable::conditional_select(
            &a.0, &b.0, choice,
        ))
    }
}

/// Constant time: delegates to [`Word`](Word).
impl ConstantTimeEq for Limb {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        self.0.ct_eq(&rhs.0)
    }
}

/// Constant time: delegates to [`Word`](Word).
impl ConstantTimeOrd for Limb {
    fn ct_lt(&self, rhs: &Self) -> Choice {
        self.0.ct_lt(&rhs.0)
    }
}

/// Overwrites the word with zero through a volatile write.
impl Zeroize for Limb {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}
