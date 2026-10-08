//! Addition of [`FixedBigUint`].

use core::ops::{Add, AddAssign};

use super::FixedBigUint;
use crate::limb::add_assign_limbs;

/// In place; panics on overflow in every build, unlike the primitive
/// integers, whose check depends on the profile. Constant time, apart
/// from that panic.
impl<const N: usize> AddAssign<&FixedBigUint<N>> for FixedBigUint<N> {
    fn add_assign(&mut self, rhs: &FixedBigUint<N>) {
        let carry = add_assign_limbs(self.limbs_mut(), rhs.as_limbs(), 0);
        assert!(carry == 0, "attempt to add with overflow");
    }
}

/// The same as `+= &rhs`. Constant time, apart from the panic on overflow.
impl<const N: usize> AddAssign<FixedBigUint<N>> for FixedBigUint<N> {
    fn add_assign(&mut self, rhs: FixedBigUint<N>) {
        *self += &rhs;
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Add<&FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn add(mut self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self += rhs;
        self
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Add<FixedBigUint<N>> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn add(mut self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        self += &rhs;
        self
    }
}

/// In the storage of `rhs`, as addition is commutative. Constant time,
/// apart from the panic on overflow.
impl<const N: usize> Add<FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn add(self, mut rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        rhs += self;
        rhs
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic
/// on overflow.
impl<const N: usize> Add<&FixedBigUint<N>> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn add(self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        self.clone() + rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::FixedBigUint;
    use crate::{Limb, LimbArray, Word};

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
                        &FixedBigUint::<8>::from(a) + &FixedBigUint::<8>::from(b),
                        FixedBigUint::<8>::from(sum),
                        "{a} {b}"
                    );
                }
            }
        }
    }

    #[test]
    fn all_six_forms_give_the_same_result() {
        let (x, y) = (
            FixedBigUint::<8>::from(255u128),
            FixedBigUint::<8>::from(0xf0f0u128),
        );
        let expected = FixedBigUint::<8>::from(255u128 + 0xf0f0);
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
    #[should_panic(expected = "attempt to add with overflow")]
    fn a_carry_out_of_the_top_limb_panics() {
        let max = FixedBigUint::<1>::new(LimbArray::new([Limb::new(Word::MAX)]));
        let _ = max + FixedBigUint::<1>::from(1u8);
    }
}
