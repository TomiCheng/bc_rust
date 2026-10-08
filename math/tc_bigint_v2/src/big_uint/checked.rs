//! Checked arithmetic of [`BigUint`]: `None` where the operators panic.

use num_traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedShl, CheckedShr, CheckedSub, Zero,
};

use super::BigUint;

/// Always `Some`, as the sum never overflows. Variable time: only for
/// public values.
impl CheckedAdd for BigUint {
    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        Some(self + rhs)
    }
}

/// `None` when `rhs` is larger, as the difference would be negative.
/// Variable time: only for public values.
impl CheckedSub for BigUint {
    fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        (rhs <= self).then(|| self - rhs)
    }
}

/// Always `Some`, as the product never overflows. Variable time: only for
/// public values.
impl CheckedMul for BigUint {
    fn checked_mul(&self, rhs: &Self) -> Option<Self> {
        Some(self * rhs)
    }
}

/// `None` when `rhs` is zero; it cannot overflow. Variable time: only for
/// public values.
impl CheckedDiv for BigUint {
    fn checked_div(&self, rhs: &Self) -> Option<Self> {
        (!rhs.is_zero()).then(|| self / rhs)
    }
}

/// `None` when `rhs` is zero; it cannot overflow. Variable time: only for
/// public values.
impl CheckedRem for BigUint {
    fn checked_rem(&self, rhs: &Self) -> Option<Self> {
        (!rhs.is_zero()).then(|| self % rhs)
    }
}

/// Always `Some`, as there is no width to shift past. Variable time: only
/// for public values.
impl CheckedShl for BigUint {
    fn checked_shl(&self, shift: u32) -> Option<Self> {
        Some(self << shift)
    }
}

/// Always `Some`, as there is no width to shift past. Variable time: only
/// for public values.
impl CheckedShr for BigUint {
    fn checked_shr(&self, shift: u32) -> Option<Self> {
        Some(self >> shift)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{
        CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedShl, CheckedShr, CheckedSub,
    };

    use crate::{BigUint, Word};

    const VALUES: [u128; 7] = [
        0,
        1,
        255,
        u64::MAX as u128,
        u64::MAX as u128 + 1,
        i128::MAX as u128,
        u128::MAX,
    ];

    const SHIFTS: [u32; 6] = [0, 1, Word::BITS, 127, 128, 200];

    #[test]
    fn only_division_by_zero_and_a_negative_difference_give_none() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigUint::from(a), BigUint::from(b));
                assert_eq!(x.checked_add(&y), Some(&x + &y), "{a} {b}");
                assert_eq!(
                    x.checked_sub(&y),
                    a.checked_sub(b).map(BigUint::from),
                    "{a} {b}"
                );
                assert_eq!(x.checked_mul(&y), Some(&x * &y), "{a} {b}");
                let nonzero = b != 0;
                assert_eq!(x.checked_div(&y), nonzero.then(|| &x / &y), "{a} {b}");
                assert_eq!(x.checked_rem(&y), nonzero.then(|| &x % &y), "{a} {b}");
            }
        }
    }

    #[test]
    fn checked_shifts_always_give_some() {
        for a in VALUES {
            for shift in SHIFTS {
                let x = BigUint::from(a);
                assert_eq!(x.checked_shl(shift), Some(&x << shift), "{a} {shift}");
                assert_eq!(x.checked_shr(shift), Some(&x >> shift), "{a} {shift}");
            }
        }
    }
}
