//! Powers of [`PaddedBigUint`].

use num_traits::{One, Pow};

use super::PaddedBigUint;
use crate::wipe::replace_wiped;

/// `self` to the power `exp` at the width of `self`, squaring from the top
/// bit of `exp` down, so that no step passes the result: panics when the
/// result does not fit the width, in every build, unlike the primitive
/// integers, whose check depends on the profile. Zero to the power zero is
/// one; at width zero it takes one limb. Constant time in `self`; `exp`
/// must be public, as its bits decide which multiplications happen.
impl Pow<u32> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn pow(self, exp: u32) -> PaddedBigUint {
        // one at the width of self
        let mut power = self.clone();
        power.set_one();
        // from the top bit of exp down, each step a power of self whose
        // exponent is a leading part of exp's bits, so at most the result
        for bit in (0..u32::BITS - exp.leading_zeros()).rev() {
            // the square takes the place of the power, which is wiped
            let squared = &power * &power;
            replace_wiped(&mut power, squared);
            if exp >> bit & 1 == 1 {
                power *= self;
            }
        }
        power
    }
}

/// The same as `(&self).pow(exp)`. Constant time in `self`; `exp` must be
/// public.
impl Pow<u32> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn pow(self, exp: u32) -> PaddedBigUint {
        (&self).pow(exp)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{One, Pow};

    use crate::{PaddedBigUint, Word};

    const BASES: [u128; 7] = [0, 1, 2, 3, 255, u64::MAX as u128, u128::MAX];

    const EXPONENTS: [u32; 8] = [0, 1, 2, 3, 7, 64, 127, 128];

    #[test]
    fn powers_that_fit_match_the_primitive_ones() {
        for a in BASES {
            for exp in EXPONENTS {
                if let Some(power) = a.checked_pow(exp) {
                    let x = PaddedBigUint::from(a);
                    assert_eq!((&x).pow(exp), PaddedBigUint::from(power), "{a} {exp}");
                    assert_eq!(x.pow(exp), PaddedBigUint::from(power), "{a} {exp}");
                }
            }
        }
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn a_power_past_the_width_panics() {
        let _ = PaddedBigUint::from(2u128).pow(128);
    }

    #[test]
    fn the_power_keeps_the_width_of_self() {
        let narrow = PaddedBigUint::from(3u8).pow(5);
        assert_eq!(narrow, PaddedBigUint::from(243u8));
        assert_eq!(narrow.as_limbs().len(), 1);
        let wide = PaddedBigUint::from(3u128).pow(5);
        assert_eq!(wide.as_limbs().len(), (u128::BITS / Word::BITS) as usize);
    }

    #[test]
    fn zero_to_the_power_zero_at_width_zero_takes_one_limb() {
        let power = PaddedBigUint::default().pow(0);
        assert!(power.is_one());
        assert_eq!(power.as_limbs().len(), 1);
    }
}
