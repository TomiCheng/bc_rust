//! Conversions into [`FixedBigUint`].

use super::FixedBigUint;
use crate::limb::split_u128_into;
use crate::{Limb, LimbArray, Word};

macro_rules! from_unsigned {
    ($($ty:ty),*) => {$(
        /// Fails to compile when `N` limbs cannot hold every value of the
        /// source type. Constant time.
        impl<const N: usize> From<$ty> for FixedBigUint<N> {
            fn from(value: $ty) -> Self {
                const {
                    assert!(
                        N * Word::BITS as usize >= <$ty>::BITS as usize,
                        "too few limbs for the source type"
                    )
                };
                let mut limbs = [Limb::new(0); N];
                split_u128_into(value as u128, 0, &mut limbs);
                Self::new(LimbArray::new(limbs))
            }
        }
    )*};
}

from_unsigned!(u8, u16, u32, u64, u128, usize);
