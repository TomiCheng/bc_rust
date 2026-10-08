//! Bitwise complement of [`PaddedBigInt`].

use core::ops::Not;

use super::PaddedBigInt;
use crate::limb::invert;

/// Every bit of the width inverted, in place, which is `-self - 1`, as for
/// the primitive integers. Constant time.
impl Not for PaddedBigInt {
    type Output = PaddedBigInt;

    fn not(mut self) -> PaddedBigInt {
        invert(self.limbs_mut());
        self
    }
}

/// Inverts a copy of `self`, so it allocates once. Constant time.
impl Not for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn not(self) -> PaddedBigInt {
        !self.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::PaddedBigInt;

    const VALUES: [i128; 9] = [
        i128::MIN,
        i64::MIN as i128 - 1,
        -129,
        -1,
        0,
        1,
        255,
        u64::MAX as i128,
        i128::MAX,
    ];

    #[test]
    fn inverting_matches_the_primitive_one() {
        for a in VALUES {
            let x = PaddedBigInt::from(a);
            assert_eq!(!&x, PaddedBigInt::from(!a), "{a}");
            assert_eq!(!!x.clone(), x, "{a}");
        }
    }

    #[test]
    fn both_forms_give_the_same_result() {
        let x = PaddedBigInt::from(-255i16);
        assert_eq!(!x.clone(), !&x);
    }

    #[test]
    fn inverting_is_negating_less_one() {
        for a in VALUES.into_iter().filter(|&a| a != i128::MIN) {
            let x = PaddedBigInt::from(a);
            assert_eq!(!&x, -&x - PaddedBigInt::from(1i8), "{a}");
        }
    }
}
