//! Exponentiation of [`BigMontyForm`].

use tc_bigint_v2::BigUint;

use super::BigMontyForm;

impl BigMontyForm<'_> {
    /// `self` to the power `exponent`, squaring from the top set bit of
    /// `exponent` down and multiplying only where a bit is set. Variable
    /// time: only for a public exponent, as that of an RSA public key is; a
    /// secret one goes through the `pow` of [`PaddedMontyForm`]. There is no
    /// constant-time `pow` here, as a `BigUint` exponent shows its length.
    ///
    /// [`PaddedMontyForm`]: crate::PaddedMontyForm
    pub fn pow_vartime(&self, exponent: &BigUint) -> Self {
        let mut power = Self::one(self.params);
        for index in (0..exponent.bits()).rev() {
            power = power.square();
            if exponent.bit(index) {
                power *= self;
            }
        }
        power
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int, mul_mod, params};

    /// `a` to the power `e`, mod `m`, by squaring from the low bit up.
    fn pow_mod(a: u64, e: u64, m: u64) -> u64 {
        let (mut base, mut power, mut e) = (a % m, 1 % m, e);
        while e != 0 {
            if e & 1 == 1 {
                power = mul_mod(power, base, m);
            }
            base = mul_mod(base, base, m);
            e >>= 1;
        }
        power
    }

    #[test]
    fn the_power_matches_squaring_and_multiplying_in_primitives() {
        for m in MODULI {
            let params = params(m);
            for a in VALUES {
                for e in VALUES {
                    let power = form(a, &params).pow_vartime(&int(e)).retrieve();
                    assert_eq!(power, int(pow_mod(a, e, m)), "{a} {e} {m}");
                }
            }
        }
    }
}
