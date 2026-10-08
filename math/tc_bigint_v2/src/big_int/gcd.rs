//! The greatest common divisor of [`BigInt`].

use super::BigInt;
use crate::Gcd;

impl BigInt {
    /// The greatest common divisor of the magnitudes, which is never negative;
    /// `gcd(0, 0)` is zero. Variable time: only for public values.
    pub fn gcd(&self, other: &Self) -> Self {
        BigInt::from(
            self.clone()
                .unsigned_abs()
                .gcd(&other.clone().unsigned_abs()),
        )
    }
}

/// Through [`BigInt::gcd`]. Variable time: only for public values.
impl Gcd for BigInt {
    type Output = Self;

    fn gcd(&self, rhs: &Self) -> Self {
        BigInt::gcd(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::BigInt;

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
                let expected = BigInt::from(reference(a.unsigned_abs(), b.unsigned_abs()));
                assert_eq!(BigInt::from(a).gcd(&BigInt::from(b)), expected, "{a} {b}");
            }
        }
    }

    #[test]
    fn a_shared_factor_of_long_values_is_found() {
        use num_traits::Pow;

        // two Mersenne primes times a factor with powers of two and three
        let factor = (BigInt::from(3u8).pow(50) << 70) * BigInt::from(-1i8);
        let m = (BigInt::from(1u8) << 127) - BigInt::from(1u8);
        let n = (BigInt::from(1u8) << 89) - BigInt::from(1u8);
        let divisor = (&factor * &m).gcd(&(&factor * &n));
        assert_eq!(divisor, BigInt::from(3u8).pow(50) << 70);
    }
}
