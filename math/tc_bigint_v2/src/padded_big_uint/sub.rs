//! Subtraction of [`PaddedBigUint`].

use core::ops::{Sub, SubAssign};

use super::PaddedBigUint;
use crate::limb::sub_assign_limbs;
use crate::ops::CloneFor;

/// In place at the wider width: the narrower operand is extended to it
/// with zeros, and the result takes it. Panics on overflow in every
/// build, unlike the primitive integers, whose check depends on the
/// profile. Constant time, apart from that panic.
impl SubAssign<&PaddedBigUint> for PaddedBigUint {
    fn sub_assign(&mut self, rhs: &PaddedBigUint) {
        self.widen(rhs.as_limbs().len());
        let borrow = sub_assign_limbs(self.limbs_mut(), rhs.as_limbs(), 0);
        assert!(borrow == 0, "attempt to subtract with overflow");
    }
}

/// The same as `-= &rhs`. Constant time, apart from the panic on overflow.
impl SubAssign<PaddedBigUint> for PaddedBigUint {
    fn sub_assign(&mut self, rhs: PaddedBigUint) {
        *self -= &rhs;
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// on overflow.
impl Sub<&PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn sub(mut self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self -= rhs;
        self
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// on overflow.
impl Sub<PaddedBigUint> for PaddedBigUint {
    type Output = PaddedBigUint;

    fn sub(mut self, rhs: PaddedBigUint) -> PaddedBigUint {
        self -= &rhs;
        self
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time, apart from the panic on overflow.
impl Sub<&PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn sub(self, rhs: &PaddedBigUint) -> PaddedBigUint {
        self.clone_for(rhs) - rhs
    }
}

/// In a copy of `self` at the wider width, so it allocates once. Constant
/// time, apart from the panic on overflow.
impl Sub<PaddedBigUint> for &PaddedBigUint {
    type Output = PaddedBigUint;

    fn sub(self, rhs: PaddedBigUint) -> PaddedBigUint {
        self.clone_for(&rhs) - &rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::PaddedBigUint;
    use crate::Word;

    const VALUES: [u128; 7] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX,
    ];

    #[test]
    fn differences_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(difference) = a.checked_sub(b) {
                    assert_eq!(
                        &PaddedBigUint::from(a) - &PaddedBigUint::from(b),
                        PaddedBigUint::from(difference),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            PaddedBigUint::from(0xf0f0u128),
            PaddedBigUint::from(255u128),
        );
        let expected = PaddedBigUint::from(0xf0f0u128 - 255);
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
    fn a_narrower_operand_is_zero_extended_to_the_wider_width() {
        let difference = PaddedBigUint::from(1u8) - PaddedBigUint::from(1u128);
        assert_eq!(difference, PaddedBigUint::from(0u8));
        assert_eq!(
            difference.as_limbs().len(),
            (u128::BITS / Word::BITS) as usize
        );
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn a_borrow_out_of_the_wider_width_panics() {
        let _ = PaddedBigUint::from(0u128) - PaddedBigUint::from(1u8);
    }
}
