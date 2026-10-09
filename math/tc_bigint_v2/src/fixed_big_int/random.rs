//! Random values of [`FixedBigInt`].

use num_traits::Zero;
use rand_core::TryRng;
use tc_zeroize::{Zeroize, Zeroizing};

use super::FixedBigInt;
use crate::limb::{add_assign_limbs, fill_bits, sub_assign_limbs};
use crate::{FixedBigUint, RandomBits, RandomRange, Word};

/// A value drawn uniformly from `[0, 2^bits)`, from the bytes of `rng`, low
/// first; the limbs above `bits` stay zero, so the value is never negative.
/// Panics when `bits` reaches the sign bit of the `N` limbs, in every
/// build. Constant time, apart from that panic: `bits` must be public.
impl<const N: usize> RandomBits for FixedBigInt<N> {
    fn try_random_bits<R: TryRng + ?Sized>(rng: &mut R, bits: u32) -> Result<Self, R::Error> {
        assert!(
            bits < N as u32 * Word::BITS,
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

/// A value drawn uniformly from `[low, high)`: the span `high - low`, as an
/// unsigned value of the `N` limbs, which holds it even when the range
/// passes the largest signed one, takes a value drawn below it as the
/// `RandomRange` of [`FixedBigUint`] draws, and `low` is added to that.
/// Panics when `low` is not below `high`, in every build. Constant time in
/// the value drawn, apart from that panic: how many draws it took shows,
/// but only the draws thrown away decide it, and the range must be public.
impl<const N: usize> RandomRange for FixedBigInt<N> {
    fn try_random_range<R: TryRng + ?Sized>(
        rng: &mut R,
        low: &Self,
        high: &Self,
    ) -> Result<Self, R::Error> {
        assert!(low < high, "attempt to draw from an empty range");
        // high - low in two's complement, read as unsigned
        let mut span = Zeroizing::new(FixedBigUint::<N>::zero());
        span.limbs_mut().copy_from_slice(high.as_limbs());
        sub_assign_limbs(span.limbs_mut(), low.as_limbs(), 0);
        let drawn = Zeroizing::new(FixedBigUint::try_random_range(
            rng,
            &FixedBigUint::zero(),
            &span,
        )?);
        let mut value = low.clone();
        // below the span, so the sum stays below `high`
        add_assign_limbs(value.limbs_mut(), drawn.as_limbs(), 0);
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::ToPrimitive;

    use crate::testing::{Broken, Counting, Failure, Xorshift, counted};
    use crate::{FixedBigInt, RandomBits, RandomRange, Word};

    /// The limbs of 128 bits, to compare against `i128`.
    const LIMBS: usize = (i128::BITS / Word::BITS) as usize;

    #[test]
    fn a_draw_takes_the_bytes_its_bits_need_and_is_never_negative() {
        let mut rng = Counting(0);
        let value = FixedBigInt::<LIMBS>::random_bits(&mut rng, 70);
        assert_eq!(
            value,
            FixedBigInt::<LIMBS>::from((counted(9) & ((1 << 70) - 1)) as i128)
        );
        // bytes 0x81 and on, with their top bits set, still give a value
        // that is not negative
        let value = FixedBigInt::<LIMBS>::random_bits(&mut Counting(0x80), 127);
        assert!(value > FixedBigInt::<LIMBS>::from(0i8));
    }

    #[test]
    #[should_panic(expected = "attempt to draw more bits than the type holds")]
    fn drawing_into_the_sign_bit_panics() {
        let _ = FixedBigInt::<LIMBS>::random_bits(&mut Counting(0), 128);
    }

    #[test]
    fn a_range_past_the_largest_signed_value_stays_inside() {
        let (low, high) = (
            FixedBigInt::<LIMBS>::from(i128::MIN),
            FixedBigInt::<LIMBS>::from(i128::MAX),
        );
        for seed in 1..50 {
            let value = FixedBigInt::<LIMBS>::random_range(&mut Xorshift(seed), &low, &high);
            assert!(low <= value && value < high, "{seed}");
        }
    }

    #[test]
    fn every_value_of_a_range_across_zero_comes_up_and_nothing_outside_it() {
        let (low, high) = (
            FixedBigInt::<LIMBS>::from(-5i8),
            FixedBigInt::<LIMBS>::from(3i8),
        );
        let mut rng = Xorshift(1);
        let mut seen = [false; 8];
        for _ in 0..300 {
            let value = FixedBigInt::<LIMBS>::random_range(&mut rng, &low, &high);
            assert!(low <= value && value < high);
            seen[(value.to_i8().unwrap() + 5) as usize] = true;
        }
        assert_eq!(seen, [true; 8]);
    }

    #[test]
    fn a_range_below_zero_stays_below_zero() {
        let (low, high) = (
            FixedBigInt::<LIMBS>::from(-9i8),
            FixedBigInt::<LIMBS>::from(-5i8),
        );
        for seed in 1..50 {
            let value = FixedBigInt::<LIMBS>::random_range(&mut Xorshift(seed), &low, &high);
            assert!(low <= value && value < high, "{seed}");
        }
    }

    #[test]
    #[should_panic(expected = "attempt to draw from an empty range")]
    fn an_empty_range_panics() {
        let (low, high) = (
            FixedBigInt::<LIMBS>::from(3i8),
            FixedBigInt::<LIMBS>::from(-3i8),
        );
        let _ = FixedBigInt::<LIMBS>::random_range(&mut Counting(0), &low, &high);
    }

    #[test]
    fn an_error_of_the_generator_comes_back() {
        assert_eq!(
            FixedBigInt::<LIMBS>::try_random_bits(&mut Broken, 70),
            Err(Failure)
        );
        let (low, high) = (
            FixedBigInt::<LIMBS>::from(-1000i16),
            FixedBigInt::<LIMBS>::from(1000i16),
        );
        assert_eq!(
            FixedBigInt::<LIMBS>::try_random_range(&mut Broken, &low, &high),
            Err(Failure)
        );
    }
}
