//! Subtraction of [`PaddedBigInt`].

use core::ops::{Sub, SubAssign};

use super::PaddedBigInt;
use crate::encoding::sign_fill;
use crate::limb::{signed_sub_overflowed, sub_assign_limbs};
use crate::ops::CloneFor;

/// In place at the wider width: the narrower operand is extended to it
/// with its sign, and the result takes it. Panics on overflow in every
/// build, unlike the primitive integers, whose check depends on the
/// profile. Constant time, apart from that panic.
impl SubAssign<&PaddedBigInt> for PaddedBigInt {
    fn sub_assign(&mut self, rhs: &PaddedBigInt) {
        self.widen(rhs.as_limbs().len());
        let (self_sign, rhs_sign) = (sign_fill(self.as_limbs()), sign_fill(rhs.as_limbs()));
        sub_assign_limbs(self.limbs_mut(), rhs.as_limbs(), rhs_sign);
        let difference_sign = sign_fill(self.as_limbs());
        assert!(
            !signed_sub_overflowed(self_sign, rhs_sign, difference_sign),
            "attempt to subtract with overflow"
        );
    }
}

/// The same as `-= &rhs`. Constant time, apart from the panic on overflow.
impl SubAssign<PaddedBigInt> for PaddedBigInt {
    fn sub_assign(&mut self, rhs: PaddedBigInt) {
        *self -= &rhs;
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// on overflow.
impl Sub<&PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn sub(mut self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self -= rhs;
        self
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// on overflow.
impl Sub<PaddedBigInt> for PaddedBigInt {
    type Output = PaddedBigInt;

    fn sub(mut self, rhs: PaddedBigInt) -> PaddedBigInt {
        self -= &rhs;
        self
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time, apart from the panic on overflow.
impl Sub<&PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn sub(self, rhs: &PaddedBigInt) -> PaddedBigInt {
        self.clone_for(rhs) - rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time, apart from the panic on overflow.
impl Sub<PaddedBigInt> for &PaddedBigInt {
    type Output = PaddedBigInt;

    fn sub(self, rhs: PaddedBigInt) -> PaddedBigInt {
        self.clone_for(&rhs) - &rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::PaddedBigInt;
    use crate::Word;

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
    fn differences_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(difference) = a.checked_sub(b) {
                    assert_eq!(
                        &PaddedBigInt::from(a) - &PaddedBigInt::from(b),
                        PaddedBigInt::from(difference),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (PaddedBigInt::from(255i128), PaddedBigInt::from(0xf0f0i128));
        let expected = PaddedBigInt::from(255i128 - 0xf0f0);
        assert_eq!(x.clone() - y.clone(), expected);
        assert_eq!(x.clone() - &y, expected);
        assert_eq!(&x - y.clone(), expected);
        assert_eq!(&x - &y, expected);
        let mut owned = x.clone();
        owned -= y.clone();
        assert_eq!(owned, expected);
        let mut borrowed = x;
        borrowed -= &y;
        assert_eq!(borrowed, expected);
    }

    #[test]
    fn a_narrower_operand_is_sign_extended_to_the_wider_width() {
        let difference = PaddedBigInt::from(-1i8) - PaddedBigInt::from(1i128);
        assert_eq!(difference, PaddedBigInt::from(-2i8));
        assert_eq!(
            difference.as_limbs().len(),
            (i128::BITS / Word::BITS) as usize
        );
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn passing_the_smallest_value_at_the_width_panics() {
        let _ = PaddedBigInt::from(i128::MIN) - PaddedBigInt::from(1i128);
    }
}
