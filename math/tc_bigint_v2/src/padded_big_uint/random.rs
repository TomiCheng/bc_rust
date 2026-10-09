//! Random values of [`PaddedBigUint`].

use alloc::vec;

use rand_core::TryRng;
use tc_zeroize::{Zeroize, Zeroizing};

use super::PaddedBigUint;
use crate::limb::fill_bits;
use crate::{Limb, RandomBits, RandomRange, Word};

/// A value drawn uniformly from `[0, 2^bits)`, from the bytes of `rng`, low
/// first, in a new buffer of the least width that holds `bits` bits.
/// Constant time: `bits` must be public.
impl RandomBits for PaddedBigUint {
    fn try_random_bits<R: TryRng + ?Sized>(rng: &mut R, bits: u32) -> Result<Self, R::Error> {
        let width = bits.div_ceil(Word::BITS) as usize;
        let mut value = Self::new(vec![Limb::new(0); width].into_boxed_slice());
        if let Err(error) = fill_bits(value.limbs_mut(), bits, rng) {
            // what was drawn before the generator failed is wiped
            value.zeroize();
            return Err(error);
        }
        Ok(value)
    }
}

/// A value drawn uniformly from `[low, high)`, at the wider of their widths:
/// a value below the span `high - low` is drawn into one buffer of that
/// width, again while it is not below it, each draw over the one before,
/// and moved up by `low`. Each draw falls below the span at least half the
/// time. Panics when `low` is not below `high`, in every build. Constant
/// time in the value drawn, apart from that panic: how many draws it took
/// shows, but only the draws thrown away decide it, and the range and the
/// widths must be public.
impl RandomRange for PaddedBigUint {
    fn try_random_range<R: TryRng + ?Sized>(
        rng: &mut R,
        low: &Self,
        high: &Self,
    ) -> Result<Self, R::Error> {
        assert!(low < high, "attempt to draw from an empty range");
        let span = Zeroizing::new(high - low);
        let mut drawn = Self::new(vec![Limb::new(0); span.as_limbs().len()].into_boxed_slice());
        loop {
            if let Err(error) = fill_bits(drawn.limbs_mut(), span.bits(), rng) {
                drawn.zeroize();
                return Err(error);
            }
            if drawn < *span {
                // below the span, so the sum stays below `high`
                drawn += low;
                return Ok(drawn);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ToPrimitive;

    use crate::testing::{Broken, Counting, Failure, Xorshift, counted};
    use crate::{PaddedBigUint, RandomBits, RandomRange, Word};

    #[test]
    fn a_draw_takes_the_bytes_its_bits_need_at_the_least_width() {
        let mut rng = Counting(0);
        let value = PaddedBigUint::random_bits(&mut rng, 70);
        assert_eq!(value, PaddedBigUint::from(counted(9) & ((1 << 70) - 1)));
        assert_eq!(value.as_limbs().len(), 70u32.div_ceil(Word::BITS) as usize);
        assert_eq!(
            PaddedBigUint::random_bits(&mut rng, 8),
            PaddedBigUint::from(10u8)
        );
        let none = PaddedBigUint::random_bits(&mut rng, 0);
        assert_eq!((none.as_limbs().len(), none), (0, PaddedBigUint::from(0u8)));
    }

    #[test]
    fn every_value_of_a_small_range_comes_up_and_nothing_outside_it() {
        let (low, high) = (PaddedBigUint::from(5u8), PaddedBigUint::from(9u8));
        let mut rng = Xorshift(1);
        let mut seen = [false; 4];
        for _ in 0..200 {
            let value = PaddedBigUint::random_range(&mut rng, &low, &high);
            assert!(low <= value && value < high);
            seen[usize::from(value.to_u8().unwrap() - 5)] = true;
        }
        assert_eq!(seen, [true; 4]);
    }

    #[test]
    fn a_draw_past_the_span_is_drawn_again() {
        let (low, high) = (PaddedBigUint::from(10u8), PaddedBigUint::from(13u8));
        let mut rng = Counting(2);
        assert_eq!(PaddedBigUint::random_range(&mut rng, &low, &high), low);
        assert_eq!(
            PaddedBigUint::random_bits(&mut rng, 8),
            PaddedBigUint::from(5u8)
        );
    }

    #[test]
    fn the_value_takes_the_wider_width_of_the_range() {
        let wide = (u128::BITS / Word::BITS) as usize;
        let (low, high) = (PaddedBigUint::from(5u8), PaddedBigUint::from(9u128));
        let value = PaddedBigUint::random_range(&mut Xorshift(2), &low, &high);
        assert_eq!(value.as_limbs().len(), wide);
    }

    #[test]
    #[should_panic(expected = "attempt to draw from an empty range")]
    fn an_empty_range_panics() {
        let (low, high) = (PaddedBigUint::from(9u8), PaddedBigUint::from(3u8));
        let _ = PaddedBigUint::random_range(&mut Counting(0), &low, &high);
    }

    #[test]
    fn an_error_of_the_generator_comes_back() {
        assert_eq!(
            PaddedBigUint::try_random_bits(&mut Broken, 70),
            Err(Failure)
        );
        let (low, high) = (PaddedBigUint::from(1u8), PaddedBigUint::from(1000u16));
        assert_eq!(
            PaddedBigUint::try_random_range(&mut Broken, &low, &high),
            Err(Failure)
        );
    }
}
