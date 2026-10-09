//! Random values of [`BigUint`].

use alloc::vec;

use rand_core::TryRng;

use super::BigUint;
use crate::limb::fill_bits;
use crate::{Limb, RandomBits, RandomRange, Word};

/// A value drawn uniformly from `[0, 2^bits)`, from the bytes of `rng`, low
/// first, and trimmed. Variable time, as the trimmed length follows the
/// value: only for public values; secret ones go through
/// [`PaddedBigUint`].
///
/// [`PaddedBigUint`]: crate::PaddedBigUint
impl RandomBits for BigUint {
    fn try_random_bits<R: TryRng + ?Sized>(rng: &mut R, bits: u32) -> Result<Self, R::Error> {
        let mut limbs = vec![Limb::new(0); bits.div_ceil(Word::BITS) as usize];
        fill_bits(&mut limbs, bits, rng)?;
        Ok(BigUint::new(limbs))
    }
}

/// A value drawn uniformly from `[low, high)`: a value below the span
/// `high - low` is drawn as `RandomBits` draws one of the bits of the span,
/// again while it is not below it, and moved up by `low`. Each draw falls
/// below the span at least half the time. Panics when `low` is not below
/// `high`, in every build. Variable time: only for public values; secret
/// ones go through [`PaddedBigUint`].
///
/// [`PaddedBigUint`]: crate::PaddedBigUint
impl RandomRange for BigUint {
    fn try_random_range<R: TryRng + ?Sized>(
        rng: &mut R,
        low: &Self,
        high: &Self,
    ) -> Result<Self, R::Error> {
        assert!(low < high, "attempt to draw from an empty range");
        let span = high - low;
        loop {
            let drawn = Self::try_random_bits(rng, span.bits())?;
            if drawn < span {
                return Ok(drawn + low);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ToPrimitive;

    use crate::testing::{Broken, Counting, Failure, Xorshift, counted};
    use crate::{BigUint, RandomBits, RandomRange, Word};

    #[test]
    fn a_draw_takes_the_bytes_its_bits_need_and_is_trimmed() {
        let mut rng = Counting(0);
        assert_eq!(
            BigUint::random_bits(&mut rng, 70),
            BigUint::from(counted(9) & ((1 << 70) - 1))
        );
        assert_eq!(BigUint::random_bits(&mut rng, 8), BigUint::from(10u8));
        // 65 bits take nine bytes, 12 to 20; the low bit of 20 is zero, so
        // the top limb is too, and is trimmed
        let value = BigUint::random_bits(&mut Counting(11), 65);
        let expected = (12..=19u8)
            .rev()
            .fold(0u128, |value, byte| value << 8 | u128::from(byte));
        assert_eq!(value, BigUint::from(expected));
        assert_eq!(value.as_limbs().len(), (u64::BITS / Word::BITS) as usize);
    }

    #[test]
    fn every_value_of_a_small_range_comes_up_and_nothing_outside_it() {
        let (low, high) = (BigUint::from(5u8), BigUint::from(9u8));
        let mut rng = Xorshift(1);
        let mut seen = [false; 4];
        for _ in 0..200 {
            let value = BigUint::random_range(&mut rng, &low, &high);
            assert!(low <= value && value < high);
            seen[usize::from(value.to_u8().unwrap() - 5)] = true;
        }
        assert_eq!(seen, [true; 4]);
    }

    #[test]
    fn a_draw_past_the_span_is_drawn_again() {
        let (low, high) = (BigUint::from(10u8), BigUint::from(13u8));
        let mut rng = Counting(2);
        assert_eq!(BigUint::random_range(&mut rng, &low, &high), low);
        assert_eq!(BigUint::random_bits(&mut rng, 8), BigUint::from(5u8));
    }

    #[test]
    #[should_panic(expected = "attempt to draw from an empty range")]
    fn an_empty_range_panics() {
        let value = BigUint::from(9u8);
        let _ = BigUint::random_range(&mut Counting(0), &value, &value);
    }

    #[test]
    fn an_error_of_the_generator_comes_back() {
        assert_eq!(BigUint::try_random_bits(&mut Broken, 70), Err(Failure));
        let (low, high) = (BigUint::from(1u8), BigUint::from(1000u16));
        assert_eq!(
            BigUint::try_random_range(&mut Broken, &low, &high),
            Err(Failure)
        );
    }
}
