//! The Montgomery parameters of a [`FixedBigUint`] modulus.

use num_traits::{WrappingSub, Zero};
use tc_bigint_v2::{FixedBigUint, Word};
use tc_zeroize::Zeroize;

use super::add::add_residues;
use crate::Odd;
use crate::monty::neg_inverse;

/// What Montgomery arithmetic modulo an odd [`FixedBigUint`] `m` needs
/// worked out once: `m` itself, `R mod m` and `R² mod m` for
/// `R = 2^(N · Word::BITS)`, and `-m⁻¹ mod 2^Word::BITS`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedMontyParams<const N: usize> {
    /// The modulus, odd as `new` took it.
    pub(crate) modulus: FixedBigUint<N>,
    /// `-m⁻¹ mod 2^Word::BITS`, from the low limb of the modulus.
    pub(crate) inverse: Word,
    /// `R mod m`, one in Montgomery form.
    pub(crate) one: FixedBigUint<N>,
    /// `R² mod m`, which takes a value into Montgomery form.
    pub(crate) r2: FixedBigUint<N>,
}

impl<const N: usize> FixedMontyParams<N> {
    /// The parameters of `modulus`. `R mod m` is the negation of `m` wrapped
    /// at the `N` limbs, reduced, and `R² mod m` is that doubled
    /// `N · Word::BITS` times. Constant time, so that a secret modulus, as
    /// the primes of an RSA key are, is safe.
    pub fn new(modulus: Odd<FixedBigUint<N>>) -> Self {
        let modulus = modulus.into_inner();
        let inverse = neg_inverse(modulus.as_limbs()[0].to_word());
        let one = FixedBigUint::zero().wrapping_sub(&modulus) % &modulus;
        let mut r2 = one.clone();
        for _ in 0..N as u32 * Word::BITS {
            r2 = add_residues(&r2, &r2, &modulus);
        }
        Self {
            modulus,
            inverse,
            one,
            r2,
        }
    }

    /// The modulus. Constant time.
    pub fn modulus(&self) -> &FixedBigUint<N> {
        &self.modulus
    }
}

/// Overwrites the modulus and every value derived from it, which leaves the
/// parameters of no use. Constant time.
impl<const N: usize> Zeroize for FixedMontyParams<N> {
    fn zeroize(&mut self) {
        self.modulus.zeroize();
        self.inverse.zeroize();
        self.one.zeroize();
        self.r2.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;
    use tc_bigint_v2::{FixedBigUint, Word};
    use tc_zeroize::Zeroize;

    use super::FixedMontyParams;
    use crate::Odd;

    /// The limbs of 64 bits, so that `R = 2^64` and `R²` fits in `u128`.
    const SMALL: usize = (u64::BITS / Word::BITS) as usize;

    /// The limbs of 128 bits.
    const LARGE: usize = (u128::BITS / Word::BITS) as usize;

    /// Odd moduli, from one up to the largest prime below `2^64`.
    const MODULI: [u64; 7] = [
        1,
        3,
        255,
        u32::MAX as u64,
        i64::MAX as u64,
        0xffff_ffff_ffff_ffc5,
        u64::MAX,
    ];

    #[test]
    fn the_parameters_hold_r_and_r_squared_reduced_by_the_modulus() {
        for m in MODULI {
            let params = FixedMontyParams::new(Odd::new(FixedBigUint::<SMALL>::from(m)).unwrap());
            // Both residues are below `m`, so they fit back into `u64`.
            let r = (1u128 << 64) % u128::from(m);
            let r2 = r * r % u128::from(m);
            assert_eq!(params.modulus, FixedBigUint::<SMALL>::from(m), "{m}");
            assert_eq!(params.one, FixedBigUint::<SMALL>::from(r as u64), "{m}");
            assert_eq!(params.r2, FixedBigUint::<SMALL>::from(r2 as u64), "{m}");
        }
    }

    #[test]
    fn the_inverse_negates_the_low_limb_of_the_modulus() {
        for m in MODULI {
            let params = FixedMontyParams::new(Odd::new(FixedBigUint::<SMALL>::from(m)).unwrap());
            let low = params.modulus.as_limbs()[0].to_word();
            assert_eq!(low.wrapping_mul(params.inverse), Word::MAX, "{m}");
        }
    }

    #[test]
    fn a_wider_radix_takes_the_width_of_the_type() {
        // 2^128 mod (2^127 - 1) is 2, and its square is 4.
        let modulus = FixedBigUint::<LARGE>::from(i128::MAX as u128);
        let params = FixedMontyParams::new(Odd::new(modulus).unwrap());
        assert_eq!(params.one, FixedBigUint::<LARGE>::from(2u8));
        assert_eq!(params.r2, FixedBigUint::<LARGE>::from(4u8));
    }

    #[test]
    fn zeroizing_clears_every_parameter() {
        let mut params =
            FixedMontyParams::new(Odd::new(FixedBigUint::<SMALL>::from(255u8)).unwrap());
        params.zeroize();
        assert!(params.modulus.is_zero() && params.one.is_zero() && params.r2.is_zero());
        assert_eq!(params.inverse, 0);
    }
}
