//! Addition of [`FixedBigUint`].

use core::ops::{Add, AddAssign};

use super::FixedBigUint;
use crate::Word;
use crate::limb::{add_assign_limbs, add_assign_word};

impl<const N: usize> FixedBigUint<N> {
    /// Adds `rhs` in place, wrapping around, and returns whether that
    /// overflowed. Constant time.
    pub(super) fn overflowing_add_assign(&mut self, rhs: &Self) -> bool {
        let carry = add_assign_limbs(self.limbs_mut(), rhs.as_limbs(), 0);
        carry != 0
    }
}

/// In place; panics on overflow in every build, unlike the primitive
/// integers, whose check depends on the profile. Constant time, apart
/// from that panic.
impl<const N: usize> AddAssign<&FixedBigUint<N>> for FixedBigUint<N> {
    fn add_assign(&mut self, rhs: &FixedBigUint<N>) {
        let overflowed = self.overflowing_add_assign(rhs);
        assert!(!overflowed, "attempt to add with overflow");
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

/// Adds the word to the low limb and carries on up, in place; panics on
/// overflow in every build, unlike the primitive integers, whose check
/// depends on the profile. Constant time, apart from that panic.
impl<const N: usize> AddAssign<u32> for FixedBigUint<N> {
    fn add_assign(&mut self, rhs: u32) {
        let carry = add_assign_word(self.limbs_mut(), Word::from(rhs));
        assert!(carry == 0, "attempt to add with overflow");
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Add<u32> for FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn add(mut self, rhs: u32) -> FixedBigUint<N> {
        self += rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Add<u32> for &FixedBigUint<N> {
    type Output = FixedBigUint<N>;

    fn add(self, rhs: u32) -> FixedBigUint<N> {
        self.clone() + rhs
    }
}

/// In the storage of `rhs`, as addition is commutative. Constant time, apart
/// from the panic on overflow.
impl<const N: usize> Add<FixedBigUint<N>> for u32 {
    type Output = FixedBigUint<N>;

    fn add(self, rhs: FixedBigUint<N>) -> FixedBigUint<N> {
        rhs + self
    }
}

/// In a copy of `rhs`, on the stack. Constant time, apart from the panic on
/// overflow.
impl<const N: usize> Add<&FixedBigUint<N>> for u32 {
    type Output = FixedBigUint<N>;

    fn add(self, rhs: &FixedBigUint<N>) -> FixedBigUint<N> {
        rhs.clone() + self
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

    mod words {
        use crate::{FixedBigUint, Word};

        /// The limbs of 128 bits, to compare against `u128`.
        const LIMBS: usize = (u128::BITS / Word::BITS) as usize;

        const VALUES: [u128; 7] = [
            0,
            1,
            255,
            u64::MAX as u128,
            u64::MAX as u128 + 1,
            i128::MAX as u128,
            u128::MAX,
        ];

        /// Words from zero up to the largest, with a prime between.
        const WORDS: [u32; 6] = [0, 1, 2, 10, 65_537, u32::MAX];

        #[test]
        fn adding_a_word_matches_the_primitive_sum() {
            for a in VALUES {
                for b in WORDS {
                    if let Some(exact) = a.checked_add(u128::from(b)) {
                        assert_eq!(
                            FixedBigUint::<LIMBS>::from(a) + b,
                            FixedBigUint::<LIMBS>::from(exact),
                            "{a} {b}"
                        );
                    }
                }
            }
        }

        #[test]
        #[should_panic(expected = "attempt to add with overflow")]
        fn a_sum_past_the_top_panics() {
            let _ = FixedBigUint::<LIMBS>::from(u128::MAX) + 1u32;
        }

        #[test]
        fn every_form_with_a_word_gives_the_same_result() {
            let (x, y) = (FixedBigUint::<LIMBS>::from(255u128), 0xf0f0u32);
            let expected = FixedBigUint::<LIMBS>::from(255u128 + 0xf0f0);
            assert_eq!(x.clone() + y, expected);
            assert_eq!(&x + y, expected);
            assert_eq!(y + x.clone(), expected);
            assert_eq!(y + &x, expected);
            let mut owned = x.clone();
            owned += y;
            assert_eq!(owned, expected);
        }
    }
}
