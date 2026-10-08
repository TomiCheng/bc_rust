//! The greatest common divisor of [`FixedBigUint`].

use super::FixedBigUint;
use crate::Gcd;
use crate::limb::gcd_assign_limbs;

impl<const N: usize> FixedBigUint<N> {
    /// The greatest common divisor of `self` and `other`; `gcd(0, 0)` is
    /// zero. Constant time: binary GCD in a fixed number of rounds, two for
    /// each bit of the `N` limbs.
    pub fn gcd(&self, other: &Self) -> FixedBigUint<N> {
        let (mut divisor, mut room) = (self.clone(), other.clone());
        gcd_assign_limbs(divisor.limbs_mut(), room.limbs_mut());
        divisor
    }
}

/// Through [`FixedBigUint::gcd`]. Constant time.
impl<const N: usize> Gcd for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn gcd(&self, rhs: &Self) -> FixedBigUint<N> {
        FixedBigUint::gcd(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::{FixedBigUint, Word};

    /// The limbs of 128 bits, to compare against `u128`.
    const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

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
                let expected = FixedBigUint::<LIMBS>::from(reference(a, b));
                assert_eq!(
                    FixedBigUint::<LIMBS>::from(a).gcd(&FixedBigUint::<LIMBS>::from(b)),
                    expected,
                    "{a} {b}"
                );
            }
        }
    }
}
