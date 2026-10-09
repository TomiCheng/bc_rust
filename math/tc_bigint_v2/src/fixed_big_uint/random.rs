//! Random values of [`FixedBigUint`].

use num_traits::Zero;
use rand_core::TryRng;
use tc_zeroize::{Zeroize, Zeroizing};

use super::FixedBigUint;
use crate::limb::fill_bits;
use crate::{RandomBits, RandomRange, Word};

/// A value drawn uniformly from `[0, 2^bits)`, from the bytes of `rng`, low
/// first; the limbs above `bits` stay zero. Panics when `bits` is past the
/// `N` limbs, in every build. Constant time, apart from that panic: `bits`
/// must be public.
impl<const N: usize> RandomBits for FixedBigUint<N> {
    fn try_random_bits<R: TryRng + ?Sized>(rng: &mut R, bits: u32) -> Result<Self, R::Error> {
        assert!(
            bits <= N as u32 * Word::BITS,
            "attempt to draw more bits than the type holds"
        );
        let mut value = Self::zero();
        if let Err(error) = fill_bits(value.limbs_mut(), bits, rng) {
            // what was drawn before the generator failed is wiped
            value.zeroize();
            return Err(error);
        }
        Ok(value)
    }
}

/// A value drawn uniformly from `[low, high)`: a value below the span
/// `high - low` is drawn as `RandomBits` draws one of the bits of the span,
/// again while it is not below it, and moved up by `low`. Each draw falls
/// below the span at least half the time, and a draw that does not is
/// wiped. Panics when `low` is not below `high`, in every build. Constant
/// time in the value drawn, apart from that panic: how many draws it took
/// shows, but only the draws thrown away decide it, and the range must be
/// public.
impl<const N: usize> RandomRange for FixedBigUint<N> {
    fn try_random_range<R: TryRng + ?Sized>(
        rng: &mut R,
        low: &Self,
        high: &Self,
    ) -> Result<Self, R::Error> {
        assert!(low < high, "attempt to draw from an empty range");
        let span = Zeroizing::new(high - low);
        loop {
            let mut drawn = Self::try_random_bits(rng, span.bits())?;
            if drawn < *span {
                // below the span, so the sum stays below `high`
                drawn += low;
                return Ok(drawn);
            }
            drawn.zeroize();
        }
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ToPrimitive;

    use crate::testing::{Broken, Counting, Failure, Xorshift, counted};
    use crate::{FixedBigUint, RandomBits, RandomRange, Word};

    /// The limbs of 128 bits, to compare against `u128`.
    const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

    #[test]
    fn a_draw_takes_the_bytes_its_bits_need_low_first_and_no_more() {
        let mut rng = Counting(0);
        let value = FixedBigUint::<LIMBS>::random_bits(&mut rng, 70);
        // nine bytes, the last cut to its low six bits
        let expected = counted(9) & ((1 << 70) - 1);
        assert_eq!(value, FixedBigUint::<LIMBS>::from(expected));
        // the next draw starts at the tenth byte
        assert_eq!(
            FixedBigUint::<LIMBS>::random_bits(&mut rng, 8),
            FixedBigUint::<LIMBS>::from(10u8)
        );
    }

    #[test]
    fn every_bit_of_the_width_and_none_can_be_drawn() {
        let mut rng = Counting(0);
        assert_eq!(
            FixedBigUint::<LIMBS>::random_bits(&mut rng, 0),
            FixedBigUint::<LIMBS>::from(0u8)
        );
        let value = FixedBigUint::<LIMBS>::random_bits(&mut rng, 128);
        assert_eq!(value, FixedBigUint::<LIMBS>::from(counted(16)));
    }

    #[test]
    #[should_panic(expected = "attempt to draw more bits than the type holds")]
    fn more_bits_than_the_type_holds_panic() {
        let _ = FixedBigUint::<LIMBS>::random_bits(&mut Counting(0), 129);
    }

    #[test]
    fn an_error_of_the_generator_comes_back() {
        assert_eq!(
            FixedBigUint::<LIMBS>::try_random_bits(&mut Broken, 70),
            Err(Failure)
        );
    }

    fn range(low: u128, high: u128) -> (FixedBigUint<LIMBS>, FixedBigUint<LIMBS>) {
        (FixedBigUint::from(low), FixedBigUint::from(high))
    }

    #[test]
    fn every_value_of_a_small_range_comes_up_and_nothing_outside_it() {
        let (low, high) = range(5, 9);
        let mut rng = Xorshift(1);
        let mut seen = [false; 4];
        for _ in 0..200 {
            let value = FixedBigUint::<LIMBS>::random_range(&mut rng, &low, &high);
            assert!(low <= value && value < high);
            seen[usize::from(value.to_u8().unwrap() - 5)] = true;
        }
        assert_eq!(seen, [true; 4]);
    }

    #[test]
    fn a_draw_past_the_span_is_wiped_and_drawn_again() {
        // a span of three takes two bits: the byte 3 is past it, and the
        // byte 4, cut to its low two bits, is zero
        let (low, high) = range(10, 13);
        let mut rng = Counting(2);
        assert_eq!(
            FixedBigUint::<LIMBS>::random_range(&mut rng, &low, &high),
            low
        );
        assert_eq!(
            FixedBigUint::<LIMBS>::random_bits(&mut rng, 8),
            FixedBigUint::<LIMBS>::from(5u8)
        );
    }

    #[test]
    fn a_range_of_one_value_gives_it_and_a_wide_one_stays_inside() {
        let (low, high) = range(7, 8);
        assert_eq!(
            FixedBigUint::<LIMBS>::random_range(&mut Xorshift(3), &low, &high),
            low
        );
        let (low, high) = range(u128::MAX / 3, u128::MAX);
        for seed in 1..50 {
            let value = FixedBigUint::<LIMBS>::random_range(&mut Xorshift(seed), &low, &high);
            assert!(low <= value && value < high, "{seed}");
        }
    }

    #[test]
    #[should_panic(expected = "attempt to draw from an empty range")]
    fn an_empty_range_panics() {
        let (low, high) = range(9, 9);
        let _ = FixedBigUint::<LIMBS>::random_range(&mut Counting(0), &low, &high);
    }

    #[test]
    fn an_error_of_the_generator_comes_back_from_a_range() {
        let (low, high) = range(1, 1000);
        assert_eq!(
            FixedBigUint::<LIMBS>::try_random_range(&mut Broken, &low, &high),
            Err(Failure)
        );
    }
}
