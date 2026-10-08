//! The Montgomery parameters of a [`BigUint`] modulus.

use num_traits::One;
use tc_bigint_v2::{BigUint, Word};
use tc_zeroize::Zeroize;

use crate::Odd;
use crate::monty::neg_inverse;

/// What Montgomery arithmetic modulo an odd [`BigUint`] `m` needs worked out
/// once: `m` itself, `R mod m` and `R² mod m` for `R = 2^(l · Word::BITS)`
/// at the number `l` of limbs `m` takes, and `-m⁻¹ mod 2^Word::BITS`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BigMontyParams {
    /// The modulus, odd as `new` took it.
    pub(crate) modulus: BigUint,
    /// `-m⁻¹ mod 2^Word::BITS`, from the low limb of the modulus.
    pub(crate) inverse: Word,
    /// `R mod m`, one in Montgomery form.
    pub(crate) one: BigUint,
    /// `R² mod m`, which takes a value into Montgomery form.
    pub(crate) r2: BigUint,
}

impl BigMontyParams {
    /// The parameters of `modulus`, worked out by shifting, multiplying and
    /// dividing. Variable time: only for a public modulus; a secret one goes
    /// through [`PaddedMontyParams`].
    ///
    /// [`PaddedMontyParams`]: crate::PaddedMontyParams
    pub fn new(modulus: Odd<BigUint>) -> Self {
        let modulus = modulus.into_inner();
        let limbs = modulus.as_limbs();
        let inverse = neg_inverse(limbs[0].to_word());
        let one = (BigUint::one() << (limbs.len() as u32 * Word::BITS)) % &modulus;
        let r2 = (&one * &one) % &modulus;
        Self {
            modulus,
            inverse,
            one,
            r2,
        }
    }

    /// The modulus. Constant time.
    pub fn modulus(&self) -> &BigUint {
        &self.modulus
    }
}

/// Overwrites the modulus and every value derived from it, which leaves the
/// parameters of no use. Constant time.
impl Zeroize for BigMontyParams {
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
    use tc_bigint_v2::{BigUint, Word};
    use tc_zeroize::Zeroize;

    use super::BigMontyParams;
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
            let params = BigMontyParams::new(Odd::new(BigUint::from(m)).unwrap());
            // The radix covers the limbs `m` takes, at most 64 bits here.
            let bits = (u64::BITS - m.leading_zeros()).div_ceil(Word::BITS) * Word::BITS;
            let r = (1u128 << bits) % u128::from(m);
            let r2 = r * r % u128::from(m);
            assert_eq!(params.modulus, BigUint::from(m), "{m}");
            assert_eq!(params.one, BigUint::from(r as u64), "{m}");
            assert_eq!(params.r2, BigUint::from(r2 as u64), "{m}");
        }
    }

    #[test]
    fn the_inverse_negates_the_low_limb_of_the_modulus() {
        for m in MODULI {
            let params = BigMontyParams::new(Odd::new(BigUint::from(m)).unwrap());
            let low = params.modulus.as_limbs()[0].to_word();
            assert_eq!(low.wrapping_mul(params.inverse), Word::MAX, "{m}");
        }
    }

    #[test]
    fn a_longer_modulus_takes_a_wider_radix() {
        // 2^128 mod (2^127 - 1) is 2, and its square is 4.
        let params = BigMontyParams::new(Odd::new(BigUint::from(i128::MAX as u128)).unwrap());
        assert_eq!(
            (params.one, params.r2),
            (BigUint::from(2u8), BigUint::from(4u8))
        );
    }

    #[test]
    fn zeroizing_clears_every_parameter() {
        let mut params = BigMontyParams::new(Odd::new(BigUint::from(255u8)).unwrap());
        params.zeroize();
        assert!(params.modulus.is_zero() && params.one.is_zero() && params.r2.is_zero());
        assert_eq!(params.inverse, 0);
    }
}
