//! The Montgomery parameters of a [`PaddedBigUint`] modulus.

use num_traits::{WrappingSub, Zero};
use tc_bigint_v2::{PaddedBigUint, Word};
use tc_zeroize::Zeroize;

use crate::Odd;
use crate::monty::{double_mod_assign, neg_inverse};
use tc_zeroize::Zeroizing;

/// What Montgomery arithmetic modulo an odd [`PaddedBigUint`] `m` needs
/// worked out once: `m` itself, `R mod m` and `R² mod m` for
/// `R = 2^(w · Word::BITS)` at the width `w` of `m`, which every value here
/// takes, and `-m⁻¹ mod 2^Word::BITS`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaddedMontyParams {
    /// The modulus, odd as `new` took it.
    pub(crate) modulus: PaddedBigUint,
    /// `-m⁻¹ mod 2^Word::BITS`, from the low limb of the modulus.
    pub(crate) inverse: Word,
    /// `R mod m`, one in Montgomery form.
    pub(crate) one: PaddedBigUint,
    /// `R² mod m`, which takes a value into Montgomery form.
    pub(crate) r2: PaddedBigUint,
}

impl PaddedMontyParams {
    /// The parameters of `modulus`. `R mod m` is the negation of `m` wrapped
    /// at its width, reduced, and `R² mod m` is that doubled
    /// `w · Word::BITS` times, in place in one buffer. Constant time, so that
    /// a secret modulus, as the primes of an RSA key are, is safe; the width
    /// is public.
    pub fn new(modulus: Odd<PaddedBigUint>) -> Self {
        let modulus = modulus.into_inner();
        let inverse = neg_inverse(modulus.as_limbs()[0].to_word());
        // Zero of width zero takes the width of the modulus here.
        // The negation of the modulus is wiped once it is reduced.
        let negated = Zeroizing::new(PaddedBigUint::zero().wrapping_sub(&modulus));
        let one = &*negated % &modulus;
        let mut r2 = one.clone().into_limbs();
        for _ in 0..modulus.as_limbs().len() as u32 * Word::BITS {
            double_mod_assign(&mut r2, modulus.as_limbs());
        }
        Self {
            modulus,
            inverse,
            one,
            r2: PaddedBigUint::new(r2),
        }
    }

    /// The modulus. Constant time.
    pub fn modulus(&self) -> &PaddedBigUint {
        &self.modulus
    }
}

/// Overwrites the modulus and every value derived from it, which leaves the
/// parameters of no use. Constant time.
impl Zeroize for PaddedMontyParams {
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
    use tc_bigint_v2::{PaddedBigUint, Word};
    use tc_zeroize::Zeroize;

    use super::PaddedMontyParams;
    use crate::Odd;

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
            let params = PaddedMontyParams::new(Odd::new(PaddedBigUint::from(m)).unwrap());
            // Both residues are below `m`, so they fit back into `u64`.
            let r = (1u128 << 64) % u128::from(m);
            let r2 = r * r % u128::from(m);
            assert_eq!(params.modulus, PaddedBigUint::from(m), "{m}");
            assert_eq!(params.one, PaddedBigUint::from(r as u64), "{m}");
            assert_eq!(params.r2, PaddedBigUint::from(r2 as u64), "{m}");
        }
    }

    #[test]
    fn the_inverse_negates_the_low_limb_of_the_modulus() {
        for m in MODULI {
            let params = PaddedMontyParams::new(Odd::new(PaddedBigUint::from(m)).unwrap());
            let low = params.modulus.as_limbs()[0].to_word();
            assert_eq!(low.wrapping_mul(params.inverse), Word::MAX, "{m}");
        }
    }

    #[test]
    fn the_radix_and_every_value_take_the_width_of_the_modulus() {
        // At 128 bits, 2^128 mod 255 is 1, as 2^8 is; at 64 bits it would
        // be the same, so the widths tell them apart.
        let wide = (u128::BITS / Word::BITS) as usize;
        let params = PaddedMontyParams::new(Odd::new(PaddedBigUint::from(255u128)).unwrap());
        assert_eq!(
            (params.one.as_limbs().len(), &params.one),
            (wide, &PaddedBigUint::from(1u8))
        );
        assert_eq!(
            (params.r2.as_limbs().len(), &params.r2),
            (wide, &PaddedBigUint::from(1u8))
        );
        // 2^128 mod (2^127 - 1) is 2, and its square is 4.
        let modulus = PaddedBigUint::from(i128::MAX as u128);
        let params = PaddedMontyParams::new(Odd::new(modulus).unwrap());
        assert_eq!(
            (params.one, params.r2),
            (PaddedBigUint::from(2u8), PaddedBigUint::from(4u8))
        );
    }

    #[test]
    fn zeroizing_clears_every_parameter() {
        let mut params = PaddedMontyParams::new(Odd::new(PaddedBigUint::from(255u8)).unwrap());
        params.zeroize();
        assert!(params.modulus.is_zero() && params.one.is_zero() && params.r2.is_zero());
        assert_eq!(params.inverse, 0);
    }
}
