//! The greatest common divisor of [`PaddedBigInt`].

use super::PaddedBigInt;
use crate::{Gcd, PaddedBigUint};
use tc_zeroize::Zeroizing;

impl PaddedBigInt {
    /// The greatest common divisor of the magnitudes, as the unsigned type of
    /// the same width, which holds even that of the most negative value and
    /// itself; `gcd(0, 0)` is zero. Constant time, as for
    /// [`PaddedBigUint::gcd`].
    pub fn gcd(&self, other: &Self) -> PaddedBigUint {
        // the magnitudes are wiped once the divisor is out
        let magnitudes = (
            Zeroizing::new(self.clone().unsigned_abs()),
            Zeroizing::new(other.clone().unsigned_abs()),
        );
        magnitudes.0.gcd(&magnitudes.1)
    }
}

/// Through [`PaddedBigInt::gcd`]. Constant time.
impl Gcd for PaddedBigInt {
    type Output = PaddedBigUint;

    fn gcd(&self, rhs: &Self) -> PaddedBigUint {
        PaddedBigInt::gcd(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::{PaddedBigInt, PaddedBigUint};

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
                let expected = PaddedBigUint::from(reference(a.unsigned_abs(), b.unsigned_abs()));
                assert_eq!(
                    PaddedBigInt::from(a).gcd(&PaddedBigInt::from(b)),
                    expected,
                    "{a} {b}"
                );
            }
        }
    }

    #[test]
    fn the_most_negative_value_has_a_divisor_that_only_fits_unsigned() {
        let min = PaddedBigInt::from(i128::MIN);
        assert_eq!(min.gcd(&min), PaddedBigUint::from(1u128 << 127));
    }
}
