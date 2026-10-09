//! Modular exponentiation of [`PaddedBigUint`].

use num_traits::{One, Zero};
use tc_bigint_v2::{PaddedBigUint, Word};
use tc_constant_time::{Choice, ConditionallySelectable};

use super::mul::mul_residues;
use crate::wipe::replace_wiped;
use crate::{ModPow, NonZero, Odd, PaddedMontyForm, PaddedMontyParams};
use tc_zeroize::Zeroizing;

/// `self` to the power `exponent`, mod `modulus`, at the wider of the
/// widths of `self` and `modulus`; the width of `exponent` sets only the
/// number of steps. An odd modulus, as the modulus of an RSA key and its
/// primes are, goes through Montgomery form, as [`PaddedMontyForm::pow`]
/// does, with the parameters worked out for this one call; an even one is
/// squared and multiplied on the residues, each step as `ModMul`
/// multiplies, far slower. Either way every bit of the width of `exponent`
/// takes a square and a multiplication, whatever its value. Constant time
/// in all three operands, apart from the parity of the modulus, which picks
/// the path and shows in the timing; the widths are public.
impl ModPow for PaddedBigUint {
    type Output = Self;

    fn mod_pow(&self, exponent: &Self, modulus: &NonZero<Self>) -> Self {
        let modulus: &Self = modulus;
        match Odd::new(modulus.clone()) {
            Some(odd) => {
                // The parameters, the forms and the power at the width of
                // the modulus are wiped once the power is out of them.
                let params = Zeroizing::new(PaddedMontyParams::new(odd));
                let base = Zeroizing::new(PaddedMontyForm::new(self, &params));
                let power = Zeroizing::new(base.pow(exponent));
                let power = Zeroizing::new(power.retrieve());
                // At the width of the modulus; zero at the width of `self`
                // widens it to the wider of the two.
                let mut zero = self.clone();
                zero.set_zero();
                zero + &*power
            }
            None => pow_residues(&Zeroizing::new(self % modulus), exponent, modulus),
        }
    }
}

/// `base` to the power `exponent`, mod `modulus`, for `base` below it and
/// at the wider width: the power is built from the top bit of `exponent`
/// down, squared at each bit and multiplied by `base` where the bit is set,
/// each step as `ModMul` multiplies. Constant time: the widths are public.
fn pow_residues(
    base: &PaddedBigUint,
    exponent: &PaddedBigUint,
    modulus: &PaddedBigUint,
) -> PaddedBigUint {
    // One at the width of the base, reduced so that a modulus of one gives
    // zero.
    let mut one = base.clone();
    one.set_one();
    let mut power = &one % modulus;
    for index in (0..exponent.as_limbs().len() as u32 * Word::BITS).rev() {
        // Each step wipes the power it squares and the product it leaves.
        let squared = mul_residues(&power, &power, modulus);
        replace_wiped(&mut power, squared);
        let product = Zeroizing::new(mul_residues(&power, base, modulus));
        power.conditional_assign(&product, Choice::from_lsb(u8::from(exponent.bit(index))));
    }
    power
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{PaddedBigUint, Word};

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
            let modulus = NonZero::new(PaddedBigUint::from(*m)).unwrap();
            for a in VALUES {
                for e in VALUES {
                    let (x, y) = (PaddedBigUint::from(a), PaddedBigUint::from(e));
                    let expected = PaddedBigUint::from(expected(a, e, *m));
                    assert_eq!(x.mod_pow(&y, &modulus), expected, "{a} {e} {m}");
                }
            }
        }
    }

    #[test]
    fn a_power_of_a_prime_minus_one_is_one_as_fermat_has_it() {
        // 2^127 - 1 is prime, so 3 to the power 2^127 - 2 is one below it.
        let prime = i128::MAX as u128;
        let modulus = NonZero::new(PaddedBigUint::from(prime)).unwrap();
        let power = PaddedBigUint::from(3u8).mod_pow(&PaddedBigUint::from(prime - 1), &modulus);
        assert_eq!(power, PaddedBigUint::from(1u8));
    }

    #[test]
    fn the_result_takes_the_wider_width_of_the_base_and_the_modulus() {
        let narrow = |value: u8| PaddedBigUint::from(value);
        let broad = |value: u8| PaddedBigUint::from(u128::from(value));
        let (one, wide) = (1, (u128::BITS / Word::BITS) as usize);
        for (a, e, m, width) in [
            (broad(7), narrow(5), narrow(9), wide),
            (narrow(7), narrow(5), broad(9), wide),
            (narrow(7), broad(5), narrow(9), one),
            (broad(7), narrow(0), narrow(9), wide),
        ] {
            let expected = if e == narrow(0) { 1u8 } else { 4 };
            let power = a.mod_pow(&e, &NonZero::new(m).unwrap());
            assert_eq!(
                (power.as_limbs().len(), power),
                (width, PaddedBigUint::from(expected))
            );
        }
    }
}
