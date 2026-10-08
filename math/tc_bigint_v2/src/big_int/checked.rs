//! Checked arithmetic of [`BigInt`]: `None` where the operators panic.

use num_traits::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedShl, CheckedShr, CheckedSub, Zero,
};

use super::BigInt;

/// Always `Some`, as the sum never overflows. Variable time: only for
/// public values.
impl CheckedAdd for BigInt {
    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        Some(self + rhs)
    }
}

/// Always `Some`, as the difference never overflows. Variable time: only
/// for public values.
impl CheckedSub for BigInt {
    fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        Some(self - rhs)
    }
}

/// Always `Some`, as the product never overflows. Variable time: only for
/// public values.
impl CheckedMul for BigInt {
    fn checked_mul(&self, rhs: &Self) -> Option<Self> {
        Some(self * rhs)
    }
}

/// `None` when `rhs` is zero; it cannot overflow. Variable time: only for
/// public values.
impl CheckedDiv for BigInt {
    fn checked_div(&self, rhs: &Self) -> Option<Self> {
        (!rhs.is_zero()).then(|| self / rhs)
    }
}

/// `None` when `rhs` is zero; it cannot overflow. Variable time: only for
/// public values.
impl CheckedRem for BigInt {
    fn checked_rem(&self, rhs: &Self) -> Option<Self> {
        (!rhs.is_zero()).then(|| self % rhs)
    }
}

/// Always `Some`, as there is no width to shift past. Variable time: only
/// for public values.
impl CheckedShl for BigInt {
    fn checked_shl(&self, shift: u32) -> Option<Self> {
        Some(self << shift)
    }
}

/// Always `Some`, as there is no width to shift past. Variable time: only
/// for public values.
impl CheckedShr for BigInt {
    fn checked_shr(&self, shift: u32) -> Option<Self> {
        Some(self >> shift)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{
        CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedShl, CheckedShr, CheckedSub,
    };

    use crate::{BigInt, Word};

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

    const SHIFTS: [u32; 6] = [0, 1, Word::BITS, 127, 128, 200];

    #[test]
    fn only_division_by_zero_gives_none() {
        for a in VALUES {
            for b in VALUES {
                let (x, y) = (BigInt::from(a), BigInt::from(b));
                assert_eq!(x.checked_add(&y), Some(&x + &y), "{a} {b}");
                assert_eq!(x.checked_sub(&y), Some(&x - &y), "{a} {b}");
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
                let x = BigInt::from(a);
                assert_eq!(x.checked_shl(shift), Some(&x << shift), "{a} {shift}");
                assert_eq!(x.checked_shr(shift), Some(&x >> shift), "{a} {shift}");
            }
        }
    }
}
