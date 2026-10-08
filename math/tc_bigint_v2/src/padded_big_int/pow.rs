//! Powers of [`PaddedBigInt`].

use num_traits::{One, Pow};

use super::PaddedBigInt;

/// `self` to the power `exp` at the width of `self`, squaring from the top
/// bit of `exp` down, so that no step passes the result: panics when the
/// result does not fit the width, in every build, unlike the primitive
/// integers, whose check depends on the profile. Zero to the power zero is
/// one; at width zero it takes one limb. Constant time in `self`; `exp`
/// must be public, as its bits decide which multiplications happen.
impl Pow<u32> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn pow(self, exp: u32) -> PaddedBigInt {
        // one at the width of self
        let mut power = self.clone();
        power.set_one();
        // from the top bit of exp down, each step a power of self whose
        // exponent is a leading part of exp's bits, so at most the result
        for bit in (0..u32::BITS - exp.leading_zeros()).rev() {
            power = &power * &power;
            if exp >> bit & 1 == 1 {
                power *= self;
            }
        }
        power
    }
}

/// The same as `(&self).pow(exp)`. Constant time in `self`; `exp` must be
/// public.
impl Pow<u32> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn pow(self, exp: u32) -> PaddedBigInt {
        (&self).pow(exp)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{One, Pow};

    use crate::{PaddedBigInt, Word};

    const BASES: [i128; 12] = [
        i128::MIN,
        -255,
        -3,
        -2,
        -1,
        0,
        1,
        2,
        3,
        255,
        u64::MAX as i128,
        i128::MAX,
    ];

    const EXPONENTS: [u32; 8] = [0, 1, 2, 3, 7, 64, 127, 128];

    #[test]
    fn powers_that_fit_match_the_primitive_ones() {
        for a in BASES {
            for exp in EXPONENTS {
                if let Some(power) = a.checked_pow(exp) {
                    let x = PaddedBigInt::from(a);
                    assert_eq!((&x).pow(exp), PaddedBigInt::from(power), "{a} {exp}");
                    assert_eq!(x.pow(exp), PaddedBigInt::from(power), "{a} {exp}");
                }
            }
        }
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn a_power_past_the_width_panics() {
        let _ = PaddedBigInt::from(3i128).pow(81);
    }

    #[test]
    fn the_most_negative_value_can_be_a_power() {
        assert_eq!(
            PaddedBigInt::from(-2i128).pow(127),
            PaddedBigInt::from(i128::MIN)
        );
    }

    #[test]
    fn the_power_keeps_the_width_of_self() {
        let narrow = PaddedBigInt::from(-3i8).pow(5);
        assert_eq!(narrow, PaddedBigInt::from(-243i16));
        assert_eq!(narrow.as_limbs().len(), 1);
        let wide = PaddedBigInt::from(3i128).pow(5);
        assert_eq!(wide.as_limbs().len(), (i128::BITS / Word::BITS) as usize);
    }

    #[test]
    fn zero_to_the_power_zero_at_width_zero_takes_one_limb() {
        let power = PaddedBigInt::default().pow(0);
        assert!(power.is_one());
        assert_eq!(power.as_limbs().len(), 1);
    }
}
