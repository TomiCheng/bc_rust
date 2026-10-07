//! Conversions into [`FixedBigInt`].

use super::FixedBigInt;
use crate::limb::split_u128_into;
use crate::{Limb, LimbArray, Word};

macro_rules! from_signed {
    ($($ty:ty),*) => {$(
        /// Sign-extends to `N` limbs; fails to compile when they cannot hold
        /// every value of the source type. Constant time.
        impl<const N: usize> From<$ty> for FixedBigInt<N> {
            fn from(value: $ty) -> Self {
                const {
                    assert!(
                        N * Word::BITS as usize >= <$ty>::BITS as usize,
                        "too few limbs for the source type"
                    )
                };
                let value = value as i128;
                // all ones for a negative value, zero otherwise, without a branch
                let fill = (value >> (i128::BITS - 1)) as Word;
                let mut limbs = [Limb::new(0); N];
                split_u128_into(value as u128, fill, &mut limbs);
                Self::new(LimbArray::new(limbs))
            }
        }
    )*};
}

macro_rules! from_unsigned {
    ($($ty:ty),*) => {$(
        /// Needs one bit more than the source type, so that a set top bit
        /// does not read as negative; fails to compile otherwise. Constant time.
        impl<const N: usize> From<$ty> for FixedBigInt<N> {
            fn from(value: $ty) -> Self {
                const {
                    assert!(
                        N * Word::BITS as usize > <$ty>::BITS as usize,
                        "too few limbs for the source type and a sign bit"
                    )
                };
                let mut limbs = [Limb::new(0); N];
                split_u128_into(value as u128, 0, &mut limbs);
                Self::new(LimbArray::new(limbs))
            }
        }
    )*};
}

from_signed!(i8, i16, i32, i64, i128, isize);
from_unsigned!(u8, u16, u32, u64, u128, usize);
