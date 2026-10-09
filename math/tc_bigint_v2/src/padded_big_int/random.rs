//! Random values of [`PaddedBigInt`].

use alloc::vec;

use num_traits::Zero;
use rand_core::TryRng;
use tc_zeroize::{Zeroize, Zeroizing};

use super::PaddedBigInt;
use crate::encoding::sign_fill;
use crate::limb::{add_assign_limbs, fill_bits, sub_assign_limbs};
use crate::{Limb, PaddedBigUint, RandomBits, RandomRange, Word};

/// A value drawn uniformly from `[0, 2^bits)`, from the bytes of `rng`, low
/// first, in a new buffer of the least width that holds `bits` bits with a
/// sign bit above them, so the value is never negative. Constant time:
/// `bits` must be public.
impl RandomBits for PaddedBigInt {
    fn try_random_bits<R: TryRng + ?Sized>(rng: &mut R, bits: u32) -> Result<Self, R::Error> {
        let width = (bits + 1).div_ceil(Word::BITS) as usize;
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
/// the span `high - low`, as an unsigned value of that width, which holds it
/// even when the range passes the largest signed one, takes a value drawn
/// below it as the `RandomRange` of [`PaddedBigUint`] draws, and `low` is
/// added to that. Panics when `low` is not below `high`, in every build.
/// Constant time in the value drawn, apart from that panic: how many draws
/// it took shows, but only the draws thrown away decide it, and the range
/// and the widths must be public.
impl RandomRange for PaddedBigInt {
    fn try_random_range<R: TryRng + ?Sized>(
        rng: &mut R,
        low: &Self,
        high: &Self,
    ) -> Result<Self, R::Error> {
        assert!(low < high, "attempt to draw from an empty range");
        // high - low in two's complement at the wider width, read as unsigned
        let mut span = high.clone_for(low).into_limbs();
        sub_assign_limbs(&mut span, low.as_limbs(), sign_fill(low.as_limbs()));
        let span = Zeroizing::new(PaddedBigUint::new(span));
        let drawn = Zeroizing::new(PaddedBigUint::try_random_range(
            rng,
            &PaddedBigUint::zero(),
            &span,
        )?);
        let mut value = low.clone_for(high);
        // below the span, so the sum stays below `high`
        add_assign_limbs(value.limbs_mut(), drawn.as_limbs(), 0);
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ToPrimitive;

    use crate::testing::{Broken, Counting, Failure, Xorshift, counted};
    use crate::{PaddedBigInt, RandomBits, RandomRange, Word};

    #[test]
    fn a_draw_takes_the_bytes_its_bits_need_with_room_for_the_sign() {
        let mut rng = Counting(0);
        let value = PaddedBigInt::random_bits(&mut rng, 70);
        assert_eq!(
            value,
            PaddedBigInt::from((counted(9) & ((1 << 70) - 1)) as i128)
        );
        assert_eq!(value.as_limbs().len(), 71u32.div_ceil(Word::BITS) as usize);
        // a whole limb of bits takes one more limb, for the sign
        let value = PaddedBigInt::random_bits(&mut Counting(0x80), Word::BITS);
        assert!(value > PaddedBigInt::from(0i8));
        assert_eq!(value.as_limbs().len(), 2);
    }

    #[test]
    fn the_value_takes_the_wider_width_of_the_range() {
        let wide = (i128::BITS / Word::BITS) as usize;
        let (low, high) = (PaddedBigInt::from(-5i8), PaddedBigInt::from(9i128));
        let value = PaddedBigInt::random_range(&mut Xorshift(2), &low, &high);
        assert_eq!(value.as_limbs().len(), wide);
    }

    #[test]
    fn a_range_past_the_largest_signed_value_stays_inside() {
        let (low, high) = (PaddedBigInt::from(i128::MIN), PaddedBigInt::from(i128::MAX));
        for seed in 1..50 {
            let value = PaddedBigInt::random_range(&mut Xorshift(seed), &low, &high);
            assert!(low <= value && value < high, "{seed}");
        }
    }

    #[test]
    fn every_value_of_a_range_across_zero_comes_up_and_nothing_outside_it() {
        let (low, high) = (PaddedBigInt::from(-5i8), PaddedBigInt::from(3i8));
        let mut rng = Xorshift(1);
        let mut seen = [false; 8];
        for _ in 0..300 {
            let value = PaddedBigInt::random_range(&mut rng, &low, &high);
            assert!(low <= value && value < high);
            seen[(value.to_i8().unwrap() + 5) as usize] = true;
        }
        assert_eq!(seen, [true; 8]);
    }

    #[test]
    fn a_range_below_zero_stays_below_zero() {
        let (low, high) = (PaddedBigInt::from(-9i8), PaddedBigInt::from(-5i8));
        for seed in 1..50 {
            let value = PaddedBigInt::random_range(&mut Xorshift(seed), &low, &high);
            assert!(low <= value && value < high, "{seed}");
        }
    }

    #[test]
    #[should_panic(expected = "attempt to draw from an empty range")]
    fn an_empty_range_panics() {
        let (low, high) = (PaddedBigInt::from(3i8), PaddedBigInt::from(-3i8));
        let _ = PaddedBigInt::random_range(&mut Counting(0), &low, &high);
    }

    #[test]
    fn an_error_of_the_generator_comes_back() {
        assert_eq!(PaddedBigInt::try_random_bits(&mut Broken, 70), Err(Failure));
        let (low, high) = (PaddedBigInt::from(-1000i16), PaddedBigInt::from(1000i16));
        assert_eq!(
            PaddedBigInt::try_random_range(&mut Broken, &low, &high),
            Err(Failure)
        );
    }
}
