//! Conversions into [`PaddedBigInt`].

use super::PaddedBigInt;
use crate::limb::split_u128;

macro_rules! from_signed {
    ($($ty:ty),*) => {$(
        /// Takes the width of the source type, sign-extended to whole limbs.
        /// Constant time.
        impl From<$ty> for PaddedBigInt {
            fn from(value: $ty) -> Self {
                Self::new(split_u128(value as i128 as u128, <$ty>::BITS).into_boxed_slice())
            }
        }
    )*};
}

from_signed!(i8, i16, i32, i64, i128, isize);
