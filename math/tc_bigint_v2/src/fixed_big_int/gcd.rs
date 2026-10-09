//! The greatest common divisor of [`FixedBigInt`].

use super::FixedBigInt;
use crate::{FixedBigUint, Gcd};
use tc_zeroize::Zeroizing;

impl<const N: usize> FixedBigInt<N> {
    /// The greatest common divisor of the magnitudes, as the unsigned type of
    /// the same width, which holds even that of the most negative value and
    /// itself; `gcd(0, 0)` is zero. Constant time, as for
    /// [`FixedBigUint::gcd`].
    pub fn gcd(&self, other: &Self) -> FixedBigUint<N> {
        // the magnitudes are wiped once the divisor is out
        let magnitudes = (
            Zeroizing::new(self.clone().unsigned_abs()),
            Zeroizing::new(other.clone().unsigned_abs()),
        );
        magnitudes.0.gcd(&magnitudes.1)
    }
}

/// Through [`FixedBigInt::gcd`]. Constant time.
impl<const N: usize> Gcd for FixedBigInt<N> {
    type Output = FixedBigUint<N>;

    fn gcd(&self, rhs: &Self) -> FixedBigUint<N> {
        FixedBigInt::gcd(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::{FixedBigInt, FixedBigUint, Word};

    /// The limbs of 128 bits, to compare against `i128`.
    const LIMBS: usize = (i128::BITS / Word::BITS) as usize;

    const VALUES: [i128; 10] = [
        i128::MIN,
        -(3 << 100),
        -18,
        -1,
        0,
        1,
        12,
        1 << 64,
        u64::MAX as i128,
        i128::MAX,
    ];

    /// Euclid's algorithm on `u128`, to compare against.
    fn reference(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    }

    #[test]
    fn the_divisor_matches_euclids() {
        for a in VALUES {
            for b in VALUES {
                let expected =
                    FixedBigUint::<LIMBS>::from(reference(a.unsigned_abs(), b.unsigned_abs()));
                assert_eq!(
                    FixedBigInt::<LIMBS>::from(a).gcd(&FixedBigInt::<LIMBS>::from(b)),
                    expected,
                    "{a} {b}"
                );
            }
        }
    }

    #[test]
    fn the_most_negative_value_has_a_divisor_that_only_fits_unsigned() {
        let min = FixedBigInt::<LIMBS>::from(i128::MIN);
        assert_eq!(min.gcd(&min), FixedBigUint::<LIMBS>::from(1u128 << 127));
    }
}
