//! Constant-time comparison of [`PaddedBigInt`].

use tc_constant_time::{Choice, ConstantTimeEq};

use super::PaddedBigInt;
use crate::encoding::sign_fill;
use crate::limb::ct_eq_extended;

/// Compares values, so a narrower operand counts as sign-extended.
/// Constant time: the widths only decide how far the comparison runs.
impl ConstantTimeEq for PaddedBigInt {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        let (left, right) = (self.as_limbs(), rhs.as_limbs());
        ct_eq_extended(left, sign_fill(left), right, sign_fill(right))
    }
}

/// Compares values at any width through [`ConstantTimeEq::ct_eq`], so `==`
/// is constant time.
impl PartialEq for PaddedBigInt {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl Eq for PaddedBigInt {}

#[cfg(test)]
mod tests {
    use tc_constant_time::ConstantTimeEq;

    use super::PaddedBigInt;
    use crate::ArrayEncoding;

    fn equal<T: ConstantTimeEq>(a: &T, b: &T) -> bool {
        a.ct_eq(b).unwrap_u8() == 1
    }

    #[test]
    fn a_narrower_operand_is_sign_extended() {
        assert!(equal(
            &PaddedBigInt::from(-1i8),
            &PaddedBigInt::from(-1i128)
        ));
        assert!(equal(&PaddedBigInt::from(5i128), &PaddedBigInt::from(5i8)));
    }

    #[test]
    fn sign_extension_is_not_mistaken_for_zeros() {
        let two_fifty_five = PaddedBigInt::from_le(&[0xffu8, 0x00]).unwrap();
        assert!(!equal(&PaddedBigInt::from(-1i8), &two_fifty_five));
        assert!(!equal(
            &PaddedBigInt::from(-1i8),
            &PaddedBigInt::from(1i128)
        ));
    }

    #[test]
    fn zero_width_equals_zero() {
        assert!(equal(&PaddedBigInt::default(), &PaddedBigInt::from(0i32)));
    }
}
