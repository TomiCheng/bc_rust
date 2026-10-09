//! Modular exponentiation of [`BigUint`].

use num_traits::One;
use tc_bigint_v2::BigUint;

use crate::{BigMontyForm, BigMontyParams, ModPow, NonZero, Odd};

/// `self` to the power `exponent`, mod `modulus`. An odd modulus goes
/// through Montgomery form, as [`BigMontyForm::pow_vartime`] does, with the
/// parameters worked out for this one call; an even one is squared and
/// multiplied with a division after every product. Variable time: only for
/// public values; secret ones go through the `ModPow` of [`PaddedBigUint`].
///
/// [`PaddedBigUint`]: tc_bigint_v2::PaddedBigUint
impl ModPow for BigUint {
    type Output = Self;

    fn mod_pow(&self, exponent: &Self, modulus: &NonZero<Self>) -> Self {
        let modulus: &Self = modulus;
        match Odd::new(modulus.clone()) {
            Some(odd) => {
                let params = BigMontyParams::new(odd);
                BigMontyForm::new(self, &params)
                    .pow_vartime(exponent)
                    .retrieve()
            }
            None => pow_residues(&(self % modulus), exponent, modulus),
        }
    }
}

/// `base` to the power `exponent`, mod `modulus`, for `base` below it: the
/// power is built from the top set bit of `exponent` down, squared at each
/// bit and multiplied by `base` where the bit is set, and reduced after
/// every product. Variable time.
fn pow_residues(base: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint {
    // Reduced, so that a modulus of one gives zero.
    let mut power = BigUint::one() % modulus;
    for index in (0..exponent.bits()).rev() {
        power = (&power * &power) % modulus;
        if exponent.bit(index) {
            power = (power * base) % modulus;
        }
    }
    power
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::BigUint;

    use crate::{ModPow, NonZero};

    /// Few values, as every power takes the same number of steps, the bits
    /// of the width squared.
    const VALUES: [u64; 6] = [0, 1, 2, u32::MAX as u64 + 1, i64::MAX as u64, u64::MAX];

    /// `a` to the power `e`, mod `m`, by squaring from the low bit up in
    /// `u128`, which holds every product of two `u64`.
    fn expected(a: u64, e: u64, m: u64) -> u64 {
        let m = u128::from(m);
        let (mut base, mut power, mut e) = (u128::from(a) % m, 1 % m, e);
        while e != 0 {
            if e & 1 == 1 {
                power = power * base % m;
            }
            base = base * base % m;
            e >>= 1;
        }
        power as u64
    }

    #[test]
    fn the_power_matches_squaring_and_multiplying_in_primitives() {
        for m in &VALUES[1..] {
            let modulus = NonZero::new(BigUint::from(*m)).unwrap();
            for a in VALUES {
                for e in VALUES {
                    let (x, y) = (BigUint::from(a), BigUint::from(e));
                    let expected = BigUint::from(expected(a, e, *m));
                    assert_eq!(x.mod_pow(&y, &modulus), expected, "{a} {e} {m}");
                }
            }
        }
    }

    #[test]
    fn a_power_of_a_prime_minus_one_is_one_as_fermat_has_it() {
        // 2^127 - 1 is prime, so 3 to the power 2^127 - 2 is one below it.
        let prime = i128::MAX as u128;
        let modulus = NonZero::new(BigUint::from(prime)).unwrap();
        let power = BigUint::from(3u8).mod_pow(&BigUint::from(prime - 1), &modulus);
        assert_eq!(power, BigUint::from(1u8));
    }
}
