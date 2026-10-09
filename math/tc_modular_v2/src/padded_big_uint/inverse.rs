//! Modular inversion of [`PaddedBigUint`].

use alloc::vec;

use num_traits::Zero;
use tc_bigint_v2::{Limb, PaddedBigUint};

use crate::inverse::{SCRATCH_ROWS, binary_inverse, safegcd_inverse};
use crate::{ModInverse, NonZero};

/// `x` with `self · x = 1 (mod modulus)`, below the modulus and at the wider
/// of the widths of `self` and `modulus`, or `None` when `self` and the
/// modulus share a factor. `self` is reduced first. An odd modulus goes
/// through safegcd; an even one, as the `λ(n)` of an RSA key is, through
/// the binary extended greatest common divisor, which is slower. Constant
/// time in both operands, apart from the parity of the modulus, which picks
/// the path and shows in the timing, and from whether there is an inverse,
/// which the `Option` shows; the widths are public.
impl ModInverse for PaddedBigUint {
    type Output = Self;

    fn mod_inverse(&self, modulus: &NonZero<Self>) -> Option<Self> {
        let modulus: &Self = modulus;
        let width = modulus.as_limbs().len();
        let reduced = self % modulus;
        // The residue fits the width of the modulus, which `%` reaches or
        // exceeds.
        let low = &reduced.as_limbs()[..width];
        let zero = vec![Limb::new(0); width].into_boxed_slice();
        let inverse = if modulus.bit(0) {
            let mut scratch = vec![0; SCRATCH_ROWS * width];
            safegcd_inverse(low, modulus.as_limbs(), &zero, &mut scratch)
        } else {
            binary_inverse(low, modulus.as_limbs(), &zero)
        };
        inverse.map(|limbs| {
            // Zero at the width of `self` widens the inverse to the wider of
            // the two.
            let mut widened = self.clone();
            widened.set_zero();
            widened + PaddedBigUint::new(limbs)
        })
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{PaddedBigUint, Word};

    use crate::{ModInverse, ModMul, NonZero};

    /// Moduli odd and even, from one up to the largest prime below `2^64`.
    const MODULI: [u64; 12] = [
        1,
        2,
        3,
        10,
        255,
        256,
        u32::MAX as u64,
        u32::MAX as u64 + 1,
        i64::MAX as u64,
        0xffff_ffff_ffff_ffc5,
        u64::MAX - 1,
        u64::MAX,
    ];

    const VALUES: [u64; 8] = [
        0,
        1,
        2,
        3,
        65_537,
        u32::MAX as u64 + 1,
        i64::MAX as u64,
        u64::MAX,
    ];

    /// The greatest common divisor of `a` and `b`.
    fn gcd(a: u64, b: u64) -> u64 {
        if b == 0 { a } else { gcd(b, a % b) }
    }

    #[test]
    fn an_inverse_exists_exactly_when_the_value_is_coprime_to_the_modulus() {
        for m in MODULI {
            let modulus = NonZero::new(PaddedBigUint::from(m)).unwrap();
            for a in VALUES {
                match PaddedBigUint::from(a).mod_inverse(&modulus) {
                    Some(inverse) => {
                        let product = PaddedBigUint::from(a).mod_mul(&inverse, &modulus);
                        assert!(inverse < PaddedBigUint::from(m), "{a} {m}");
                        assert_eq!(product, PaddedBigUint::from(1 % m), "{a} {m}");
                    }
                    None => assert_ne!(gcd(a % m, m), 1, "{a} {m}"),
                }
            }
        }
    }

    #[test]
    fn known_inverses_come_out() {
        let inverse = |a: u64, m: u64| {
            PaddedBigUint::from(a).mod_inverse(&NonZero::new(PaddedBigUint::from(m)).unwrap())
        };
        assert_eq!(inverse(3, 7), Some(PaddedBigUint::from(5u8)));
        assert_eq!(inverse(3, 10), Some(PaddedBigUint::from(7u8)));
        assert_eq!(inverse(0, 1), Some(PaddedBigUint::from(0u8)));
        assert_eq!(inverse(4, 10), None);
        // 3 · (2^128 - 1) / 3 is 2 · (2^127 - 1) + 1.
        let prime = NonZero::new(PaddedBigUint::from(i128::MAX as u128)).unwrap();
        let wide = PaddedBigUint::from(3u8).mod_inverse(&prime);
        assert_eq!(wide, Some(PaddedBigUint::from(u128::MAX / 3)));
    }

    #[test]
    fn the_inverse_takes_the_wider_width_of_the_value_and_the_modulus() {
        let (narrow, wide) = (
            (u64::BITS / Word::BITS) as usize,
            (u128::BITS / Word::BITS) as usize,
        );
        let at = |value: u8, wide: bool| {
            if wide {
                PaddedBigUint::from(u128::from(value))
            } else {
                PaddedBigUint::from(u64::from(value))
            }
        };
        for (value, modulus, width) in [
            (at(3, true), at(7, false), wide),
            (at(3, false), at(7, true), wide),
            (at(3, false), at(7, false), narrow),
        ] {
            let inverse = value.mod_inverse(&NonZero::new(modulus).unwrap()).unwrap();
            assert_eq!(
                (inverse.as_limbs().len(), inverse),
                (width, PaddedBigUint::from(5u8))
            );
        }
    }
}
