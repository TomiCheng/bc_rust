//! Modular exponentiation of [`FixedBigUint`].

use num_traits::One;
use tc_bigint_v2::{FixedBigUint, Word};
use tc_constant_time::{Choice, ConditionallySelectable};

use super::mul::mul_residues;
use crate::{ModPow, NonZero};

/// `self` to the power `exponent`, mod `modulus`. `self` is reduced first;
/// the power is then built from the top bit of `exponent` down, squared at
/// each bit and multiplied by `self` where the bit is set, each step as
/// `ModMul` multiplies. Every bit of the `N` limbs of `exponent` takes a
/// square and a multiplication, whatever its value. Constant time in all
/// three operands.
impl<const N: usize> ModPow for FixedBigUint<N> {
    type Output = Self;

    fn mod_pow(&self, exponent: &Self, modulus: &NonZero<Self>) -> Self {
        let modulus: &Self = modulus;
        let base = self % modulus;
        // Reduced, so that a modulus of one gives zero.
        let mut power = Self::one() % modulus;
        for index in (0..N as u32 * Word::BITS).rev() {
            power = mul_residues(&power, &power, modulus);
            let product = mul_residues(&power, &base, modulus);
            power.conditional_assign(&product, Choice::from_lsb(u8::from(exponent.bit(index))));
        }
        power
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{FixedBigUint, Word};

    use crate::{ModPow, NonZero};

    /// The limbs of 64 bits, to compare against `u64`.
    const SMALL: usize = (u64::BITS / Word::BITS) as usize;

    /// The limbs of 128 bits.
    const LARGE: usize = (u128::BITS / Word::BITS) as usize;

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
            let modulus = NonZero::new(FixedBigUint::<SMALL>::from(*m)).unwrap();
            for a in VALUES {
                for e in VALUES {
                    let (x, y) = (
                        FixedBigUint::<SMALL>::from(a),
                        FixedBigUint::<SMALL>::from(e),
                    );
                    let expected = FixedBigUint::<SMALL>::from(expected(a, e, *m));
                    assert_eq!(x.mod_pow(&y, &modulus), expected, "{a} {e} {m}");
                }
            }
        }
    }

    #[test]
    fn a_power_of_a_prime_minus_one_is_one_as_fermat_has_it() {
        // 2^127 - 1 is prime, so 3 to the power 2^127 - 2 is one below it.
        let prime = i128::MAX as u128;
        let modulus = NonZero::new(FixedBigUint::<LARGE>::from(prime)).unwrap();
        let power = FixedBigUint::<LARGE>::from(3u8)
            .mod_pow(&FixedBigUint::<LARGE>::from(prime - 1), &modulus);
        assert_eq!(power, FixedBigUint::<LARGE>::from(1u8));
    }
}
