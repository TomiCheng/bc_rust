//! Powers of [`FixedBigUint`].

use num_traits::{One, Pow};

use super::FixedBigUint;

/// `self` to the power `exp`, squaring from the top bit of `exp` down, so
/// that no step passes the result: panics when the result does not fit the
/// `N` limbs, in every build, unlike the primitive integers, whose check
/// depends on the profile. Zero to the power zero is one. Constant time in
/// `self`; `exp` must be public, as its bits decide which multiplications
/// happen.
impl<const N: usize> Pow<u32> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn pow(self, exp: u32) -> FixedBigUint<N> {
        let mut power = FixedBigUint::one();
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
impl<const N: usize> Pow<u32> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn pow(self, exp: u32) -> FixedBigUint<N> {
        (&self).pow(exp)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Pow;

    use crate::{FixedBigUint, Word};

    /// The limbs of 128 bits, to compare against `u128`.
    const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

    const BASES: [u128; 7] = [0, 1, 2, 3, 255, u64::MAX as u128, u128::MAX];

    const EXPONENTS: [u32; 8] = [0, 1, 2, 3, 7, 64, 127, 128];

    #[test]
    fn powers_that_fit_match_the_primitive_ones() {
        for a in BASES {
            for exp in EXPONENTS {
                if let Some(power) = a.checked_pow(exp) {
                    let x = FixedBigUint::<LIMBS>::from(a);
                    assert_eq!(
                        (&x).pow(exp),
                        FixedBigUint::<LIMBS>::from(power),
                        "{a} {exp}"
                    );
                    assert_eq!(x.pow(exp), FixedBigUint::<LIMBS>::from(power), "{a} {exp}");
                }
            }
        }
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn a_power_past_the_width_panics() {
        let _ = FixedBigUint::<LIMBS>::from(2u8).pow(128);
    }
}
