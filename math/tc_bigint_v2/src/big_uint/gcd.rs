//! The greatest common divisor of [`BigUint`].

use super::BigUint;
use crate::Gcd;

impl BigUint {
    /// The greatest common divisor of `self` and `other`; `gcd(0, 0)` is zero.
    /// Variable time: only for public values. Binary GCD, which halves and
    /// subtracts in place rather than dividing.
    pub fn gcd(&self, other: &Self) -> Self {
        let (Some(a_zeros), Some(b_zeros)) = (self.trailing_zeros(), other.trailing_zeros()) else {
            // one of them is zero, so the other is the divisor
            return self | other;
        };
        let (mut a, mut b) = (self >> a_zeros, other >> b_zeros);
        // both odd: take the smaller out of the larger, and halve what is
        // left until it is odd again, until it is gone
        loop {
            if a < b {
                core::mem::swap(&mut a, &mut b);
            }
            a -= &b;
            let Some(zeros) = a.trailing_zeros() else {
                break;
            };
            a >>= zeros;
        }
        b << a_zeros.min(b_zeros)
    }
}

/// Through [`BigUint::gcd`]. Variable time: only for public values.
impl Gcd for BigUint {
    type Output = Self;

    fn gcd(&self, rhs: &Self) -> Self {
        BigUint::gcd(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::BigUint;

    const VALUES: [u128; 10] = [
        0,
        1,
        12,
        18,
        255,
        1 << 64,
        3 << 100,
        u64::MAX as u128,
        i128::MAX as u128,
        u128::MAX,
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
                let expected = BigUint::from(reference(a, b));
                assert_eq!(BigUint::from(a).gcd(&BigUint::from(b)), expected, "{a} {b}");
            }
        }
    }

    #[test]
    fn a_shared_factor_of_long_values_is_found() {
        use num_traits::Pow;

        // two Mersenne primes times a factor with powers of two and three
        let factor = BigUint::from(3u8).pow(50) << 70;
        let m = (BigUint::from(1u8) << 127) - BigUint::from(1u8);
        let n = (BigUint::from(1u8) << 89) - BigUint::from(1u8);
        let divisor = (&factor * &m).gcd(&(&factor * &n));
        assert_eq!(divisor, BigUint::from(3u8).pow(50) << 70);
    }
}
