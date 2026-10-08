//! Addition of [`BigInt`].

use super::BigInt;
use crate::Limb;
use crate::encoding::sign_fill;
use crate::limb::add_assign_limbs;
use crate::ops::forward_commutative_binop;

/// In the storage of the left operand, which grows only when the right
/// one is longer or the sum needs one more limb, so it cannot overflow;
/// the result is trimmed. Variable time: only for public values.
impl core::ops::AddAssign<&BigInt> for BigInt {
    fn add_assign(&mut self, rhs: &BigInt) {
        let mut limbs = core::mem::take(self).into_limbs();
        let (self_fill, rhs_fill) = (sign_fill(&limbs), sign_fill(rhs.as_limbs()));
        limbs.resize(limbs.len().max(rhs.as_limbs().len()), Limb::new(self_fill));
        let carry = add_assign_limbs(&mut limbs, rhs.as_limbs(), rhs_fill);
        // the limb above, as if both operands were one limb longer; it is
        // needed unless it only repeats the sign of the sum below it
        let top = self_fill.wrapping_add(rhs_fill).wrapping_add(carry);
        if top != sign_fill(&limbs) {
            limbs.push(Limb::new(top));
        }
        *self = BigInt::new(limbs);
    }
}

forward_commutative_binop!(Add, add, AddAssign, add_assign, [] BigInt);

#[cfg(test)]
mod tests {
    use crate::BigInt;

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
    fn sums_that_fit_match_the_primitive_ones() {
        for a in VALUES {
            for b in VALUES {
                if let Some(sum) = a.checked_add(b) {
                    assert_eq!(
                        &BigInt::from(a) + &BigInt::from(b),
                        BigInt::from(sum),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (BigInt::from(255i128), BigInt::from(0xf0f0i128));
        let expected = BigInt::from(255i128 + 0xf0f0);
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
    fn a_sum_past_either_operand_grows_the_value() {
        let half = BigInt::from(1u128 << 127);
        assert_eq!(
            BigInt::from(i128::MIN) + BigInt::from(i128::MIN),
            -(&half + &half)
        );
    }

    #[test]
    fn opposite_values_sum_to_an_empty_zero() {
        assert!(
            (BigInt::from(-300i16) + BigInt::from(300i16))
                .as_limbs()
                .is_empty()
        );
    }
}
