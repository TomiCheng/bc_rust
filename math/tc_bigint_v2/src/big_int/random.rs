//! Random values of [`BigInt`].

use alloc::vec;

use num_traits::Zero;
use rand_core::TryRng;

use super::BigInt;
use crate::limb::fill_bits;
use crate::{BigUint, Limb, RandomBits, RandomRange, Word};

/// A value drawn uniformly from `[0, 2^bits)`, from the bytes of `rng`, low
/// first, with a sign bit above them, so it is never negative, and trimmed.
/// Variable time, as the trimmed length follows the value: only for public
/// values; secret ones go through [`PaddedBigInt`].
///
/// [`PaddedBigInt`]: crate::PaddedBigInt
impl RandomBits for BigInt {
    fn try_random_bits<R: TryRng + ?Sized>(rng: &mut R, bits: u32) -> Result<Self, R::Error> {
        let mut limbs = vec![Limb::new(0); (bits + 1).div_ceil(Word::BITS) as usize];
        fill_bits(&mut limbs, bits, rng)?;
        Ok(BigInt::new(limbs))
    }
}

/// A value drawn uniformly from `[low, high)`: the span `high - low` takes a
/// value drawn below it as the `RandomRange` of [`BigUint`] draws, and `low`
/// is added to that. Panics when `low` is not below `high`, in every build.
/// Variable time: only for public values; secret ones go through
/// [`PaddedBigInt`].
///
/// [`PaddedBigInt`]: crate::PaddedBigInt
impl RandomRange for BigInt {
    fn try_random_range<R: TryRng + ?Sized>(
        rng: &mut R,
        low: &Self,
        high: &Self,
    ) -> Result<Self, R::Error> {
        assert!(low < high, "attempt to draw from an empty range");
        let span = BigUint::try_from(high - low).expect("the span of a range is positive");
        let drawn = BigUint::try_random_range(rng, &BigUint::zero(), &span)?;
        Ok(BigInt::from(drawn) + low)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ToPrimitive;

    use crate::testing::{Broken, Counting, Failure, Xorshift, counted};
    use crate::{BigInt, RandomBits, RandomRange};

    #[test]
    fn a_draw_takes_the_bytes_its_bits_need_and_is_never_negative() {
        let mut rng = Counting(0);
        assert_eq!(
            BigInt::random_bits(&mut rng, 70),
            BigInt::from((counted(9) & ((1 << 70) - 1)) as i128)
        );
        // bytes 0x81 to 0x88 fill 64 bits with the top one set, and still
        // give a value that is not negative
        let value = BigInt::random_bits(&mut Counting(0x80), 64);
        let expected = (0x81..=0x88u8)
            .rev()
            .fold(0u128, |value, byte| value << 8 | u128::from(byte));
        assert_eq!(value, BigInt::from(expected as i128));
    }

    #[test]
    fn a_wide_range_stays_inside() {
        let (low, high) = (
            BigInt::from(i128::MIN) * BigInt::from(4i8),
            BigInt::from(i128::MAX),
        );
        for seed in 1..50 {
            let value = BigInt::random_range(&mut Xorshift(seed), &low, &high);
            assert!(low <= value && value < high, "{seed}");
        }
    }

    #[test]
    fn every_value_of_a_range_across_zero_comes_up_and_nothing_outside_it() {
        let (low, high) = (BigInt::from(-5i8), BigInt::from(3i8));
        let mut rng = Xorshift(1);
        let mut seen = [false; 8];
        for _ in 0..300 {
            let value = BigInt::random_range(&mut rng, &low, &high);
            assert!(low <= value && value < high);
            seen[(value.to_i8().unwrap() + 5) as usize] = true;
        }
        assert_eq!(seen, [true; 8]);
    }

    #[test]
    fn a_range_below_zero_stays_below_zero() {
        let (low, high) = (BigInt::from(-9i8), BigInt::from(-5i8));
        for seed in 1..50 {
            let value = BigInt::random_range(&mut Xorshift(seed), &low, &high);
            assert!(low <= value && value < high, "{seed}");
        }
    }

    #[test]
    #[should_panic(expected = "attempt to draw from an empty range")]
    fn an_empty_range_panics() {
        let (low, high) = (BigInt::from(3i8), BigInt::from(-3i8));
        let _ = BigInt::random_range(&mut Counting(0), &low, &high);
    }

    #[test]
    fn an_error_of_the_generator_comes_back() {
        assert_eq!(BigInt::try_random_bits(&mut Broken, 70), Err(Failure));
        let (low, high) = (BigInt::from(-1000i16), BigInt::from(1000i16));
        assert_eq!(
            BigInt::try_random_range(&mut Broken, &low, &high),
            Err(Failure)
        );
    }
}
