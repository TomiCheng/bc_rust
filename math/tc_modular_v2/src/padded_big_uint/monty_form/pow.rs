//! Exponentiation of [`PaddedMontyForm`].

use tc_bigint_v2::{PaddedBigUint, Word};
use tc_constant_time::{Choice, ConditionallySelectable};

use super::PaddedMontyForm;
use crate::wipe::replace_wiped;
use tc_zeroize::Zeroizing;

impl PaddedMontyForm<'_> {
    /// `self` to the power `exponent`: from the top bit of the width of
    /// `exponent` down, the power is squared and multiplied by `self` at
    /// every bit, and the product kept where the bit is set, through a
    /// constant-time select. Constant time in both operands, for a secret
    /// exponent: the widths are public.
    pub fn pow(&self, exponent: &PaddedBigUint) -> Self {
        let mut power = Self::one(self.params);
        for index in (0..exponent.as_limbs().len() as u32 * Word::BITS).rev() {
            // The square takes the place of the power, which is wiped.
            let squared = power.square();
            replace_wiped(&mut power, squared);
            let product = Zeroizing::new(&power * self);
            let bit = Choice::from_lsb(u8::from(exponent.bit(index)));
            power.value.conditional_assign(&product.value, bit);
        }
        power
    }

    /// `self` to the power `exponent`, squaring from the top set bit of
    /// `exponent` down and multiplying only where a bit is set. Variable
    /// time: only for a public exponent, as that of an RSA public key is; a
    /// secret one goes through [`pow`](Self::pow).
    pub fn pow_vartime(&self, exponent: &PaddedBigUint) -> Self {
        let mut power = Self::one(self.params);
        for index in (0..exponent.bits()).rev() {
            // The square takes the place of the power, which is wiped.
            let squared = power.square();
            replace_wiped(&mut power, squared);
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
    fn both_powers_match_squaring_and_multiplying_in_primitives() {
        for m in MODULI {
            let params = params(m);
            for a in VALUES {
                for e in VALUES {
                    let (base, expected) = (form(a, &params), int(pow_mod(a, e, m)));
                    assert_eq!(base.pow(&int(e)).retrieve(), expected, "{a} {e} {m}");
                    assert_eq!(
                        base.pow_vartime(&int(e)).retrieve(),
                        expected,
                        "{a} {e} {m}"
                    );
                }
            }
        }
    }
}
