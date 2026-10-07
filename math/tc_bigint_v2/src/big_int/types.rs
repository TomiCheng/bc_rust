use alloc::vec::Vec;

use crate::{Limb, Word};

/// Arbitrary-precision signed integer.
#[derive(Clone, Default)]
pub struct BigInt {
    limbs: Vec<Limb>,
}

impl BigInt {
    /// Takes two's-complement limbs, least significant first, and drops the
    /// leading limbs that only repeat the sign; zero becomes no limbs.
    ///
    /// Variable time: only for public values. For secrets use
    /// [`PaddedBigInt::new`](crate::PaddedBigInt::new), which keeps every limb.
    pub fn new(mut limbs: Vec<Limb>) -> Self {
        trim(&mut limbs);
        Self { limbs }
    }

    /// The limbs, least significant first, already trimmed. Constant time.
    pub fn as_limbs(&self) -> &[Limb] {
        &self.limbs
    }

    /// Unwraps the trimmed limbs, least significant first. Constant time.
    pub fn into_limbs(self) -> Vec<Limb> {
        self.limbs
    }
}

/// Drops each top limb that only repeats the sign bit of the limb below it,
/// then a lone zero limb. Variable time.
fn trim(limbs: &mut Vec<Limb>) {
    loop {
        let redundant = match limbs.as_slice() {
            [.., below, top] => {
                let below_negative = below.to_word() >> (Word::BITS - 1) == 1;
                let top = top.to_word();
                (top == 0 && !below_negative) || (top == Word::MAX && below_negative)
            }
            [top] => top.to_word() == 0,
            [] => false,
        };
        if !redundant {
            break;
        }
        limbs.pop();
    }
}
