//! Conversions into [`FixedBigUint`].

use num_traits::FromPrimitive;

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

/// `value` in `N` limbs, or `None` when it does not fit. Variable time.
fn from_wide<const N: usize>(value: u128) -> Option<FixedBigUint<N>> {
    let bits = N * Word::BITS as usize;
    if bits < u128::BITS as usize && value >> bits != 0 {
        return None;
    }
    let mut limbs = [Limb::new(0); N];
    split_u128_into(value, 0, &mut limbs);
    Some(FixedBigUint::new(LimbArray::new(limbs)))
}

/// Values that do not fit in `N` limbs, and negative values, give `None`.
/// Variable time: only for public values.
impl<const N: usize> FromPrimitive for FixedBigUint<N> {
    fn from_i64(n: i64) -> Option<Self> {
        Self::from_i128(n.into())
    }

    fn from_u64(n: u64) -> Option<Self> {
        from_wide(n.into())
    }

    fn from_i128(n: i128) -> Option<Self> {
        from_wide(u128::try_from(n).ok()?)
    }

    fn from_u128(n: u128) -> Option<Self> {
        from_wide(n)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::FromPrimitive;

    use super::FixedBigUint;
    use crate::{Limb, Word};

    #[test]
    fn limbs_above_the_value_are_zero() {
        let spanned = (u128::BITS / Word::BITS) as usize;
        let value = FixedBigUint::<8>::from(u128::MAX);
        assert!(
            value.as_limbs()[..spanned]
                .iter()
                .all(|limb| *limb == Limb::new(Word::MAX))
        );
        assert!(
            value.as_limbs()[spanned..]
                .iter()
                .all(|limb| *limb == Limb::new(0))
        );
    }

    #[test]
    fn a_single_limb_holds_a_byte() {
        assert_eq!(FixedBigUint::<1>::from(7u8).as_limbs(), [Limb::new(7)]);
    }

    #[test]
    fn from_primitive_rejects_values_too_wide_for_n_limbs() {
        assert!(FixedBigUint::<1>::from_u128(u128::MAX).is_none());
        assert_eq!(
            FixedBigUint::<1>::from_u64(u64::MAX).is_some(),
            Word::BITS == 64
        );
        assert!(FixedBigUint::<0>::from_u8(0).is_some());
        assert!(FixedBigUint::<0>::from_u8(1).is_none());
    }

    #[test]
    fn from_primitive_rejects_negative_values() {
        assert!(FixedBigUint::<4>::from_i8(-1).is_none());
        assert!(FixedBigUint::<4>::from_i128(i128::MIN).is_none());
        assert_eq!(
            FixedBigUint::<1>::from_i32(9).unwrap().as_limbs(),
            [Limb::new(9)]
        );
    }
}
