//! The greatest common divisor of [`PaddedBigUint`].

use super::PaddedBigUint;
use crate::Gcd;
use crate::limb::gcd_assign_limbs;

impl PaddedBigUint {
    /// The greatest common divisor of `self` and `other`, at the wider width;
    /// `gcd(0, 0)` is zero. Constant time: binary GCD in a fixed number of
    /// rounds, two for each bit of the wider width.
    pub fn gcd(&self, other: &Self) -> PaddedBigUint {
        let (mut divisor, mut room) = (self.clone_for(other), other.clone_for(self));
        gcd_assign_limbs(divisor.limbs_mut(), room.limbs_mut());
        divisor
    }
}

/// Through [`PaddedBigUint::gcd`]. Constant time.
impl Gcd for PaddedBigUint {
    type Output = PaddedBigUint;

    fn gcd(&self, rhs: &Self) -> PaddedBigUint {
        PaddedBigUint::gcd(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::{PaddedBigUint, Word};

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
                let expected = PaddedBigUint::from(reference(a, b));
                assert_eq!(
                    PaddedBigUint::from(a).gcd(&PaddedBigUint::from(b)),
                    expected,
                    "{a} {b}"
                );
            }
        }
    }

    #[test]
    fn the_divisor_takes_the_wider_width() {
        let divisor = PaddedBigUint::from(12u8).gcd(&PaddedBigUint::from(18u128));
        assert_eq!(divisor, PaddedBigUint::from(6u8));
        assert_eq!(divisor.as_limbs().len(), (u128::BITS / Word::BITS) as usize);
    }
}
