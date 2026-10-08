//! Addition of [`BigUint`].

use core::ops::{Add, AddAssign};

use super::BigUint;
use crate::Limb;
use crate::limb::add_assign_limbs;

/// In the storage of the left operand, which grows only when the right
/// one is longer or the sum needs one more limb, so it cannot overflow;
/// the result is trimmed. Variable time: only for public values.
impl AddAssign<&BigUint> for BigUint {
    fn add_assign(&mut self, rhs: &BigUint) {
        let mut limbs = core::mem::take(self).into_limbs();
        limbs.resize(limbs.len().max(rhs.as_limbs().len()), Limb::new(0));
        let carry = add_assign_limbs(&mut limbs, rhs.as_limbs(), 0);
        if carry != 0 {
            limbs.push(Limb::new(carry));
        }
        *self = BigUint::new(limbs);
    }
}

/// The same as `+= &rhs`. Variable time: only for public values.
impl AddAssign<BigUint> for BigUint {
    fn add_assign(&mut self, rhs: BigUint) {
        *self += &rhs;
    }
}

/// In the storage of `self`, as `+=`. Variable time: only for public
/// values.
impl Add<&BigUint> for BigUint {
    type Output = BigUint;

    fn add(mut self, rhs: &BigUint) -> BigUint {
        self += rhs;
        self
    }
}

/// In the storage of `self`, as `+=`. Variable time: only for public
/// values.
impl Add<BigUint> for BigUint {
    type Output = BigUint;

    fn add(mut self, rhs: BigUint) -> BigUint {
        self += &rhs;
        self
    }
}

/// In the storage of `rhs`, as addition is commutative. Variable time: only
/// for public values.
impl Add<BigUint> for &BigUint {
    type Output = BigUint;

    fn add(self, mut rhs: BigUint) -> BigUint {
        rhs += self;
        rhs
    }
}

/// In a copy of `self` with room for the sum, so it allocates once.
/// Variable time: only for public values.
impl Add<&BigUint> for &BigUint {
    type Output = BigUint;

    fn add(self, rhs: &BigUint) -> BigUint {
        self.clone_for(rhs) + rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::ArrayEncoding;
    use crate::BigUint;

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
    fn sums_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(sum) = a.checked_add(b) {
                    assert_eq!(
                        &BigUint::from(a) + &BigUint::from(b),
                        BigUint::from(sum),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (BigUint::from(255u128), BigUint::from(0xf0f0u128));
        let expected = BigUint::from(255u128 + 0xf0f0);
        assert_eq!(x.clone() + y.clone(), expected);
        assert_eq!(x.clone() + &y, expected);
        assert_eq!(&x + y.clone(), expected);
        assert_eq!(&x + &y, expected);
        let mut owned = x.clone();
        owned += y.clone();
        assert_eq!(owned, expected);
        let mut borrowed = x;
        borrowed += &y;
        assert_eq!(borrowed, expected);
    }

    #[test]
    fn a_carry_out_of_the_top_limb_grows_the_value() {
        let mut bytes = [0u8; 17];
        bytes[16] = 1;
        assert_eq!(
            BigUint::from(u128::MAX) + BigUint::from(1u8),
            BigUint::from_le(&bytes).unwrap()
        );
    }
}
