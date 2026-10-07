//! Conversions into [`FixedBigInt`].

use num_traits::FromPrimitive;

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

/// `value` sign-extended to `N` limbs, or `None` when it does not fit.
/// Variable time.
fn from_wide<const N: usize>(value: i128) -> Option<FixedBigInt<N>> {
    let bits = N * Word::BITS as usize;
    if bits < i128::BITS as usize {
        // the bits above the sign bit must all repeat it
        let fits = match bits {
            0 => value == 0,
            _ => matches!(value >> (bits - 1), 0 | -1),
        };
        if !fits {
            return None;
        }
    }
    let fill = (value >> (i128::BITS - 1)) as Word;
    let mut limbs = [Limb::new(0); N];
    split_u128_into(value as u128, fill, &mut limbs);
    Some(FixedBigInt::new(LimbArray::new(limbs)))
}

/// Values that do not fit in `N` limbs as two's complement give `None`.
/// Variable time: only for public values.
impl<const N: usize> FromPrimitive for FixedBigInt<N> {
    fn from_i64(n: i64) -> Option<Self> {
        from_wide(n.into())
    }

    fn from_u64(n: u64) -> Option<Self> {
        from_wide(n.into())
    }

    fn from_i128(n: i128) -> Option<Self> {
        from_wide(n)
    }

    fn from_u128(n: u128) -> Option<Self> {
        let bits = N * Word::BITS as usize;
        // above 128 bits the sign bit is always clear
        if bits > u128::BITS as usize {
            let mut limbs = [Limb::new(0); N];
            split_u128_into(n, 0, &mut limbs);
            return Some(Self::new(LimbArray::new(limbs)));
        }
        from_wide(i128::try_from(n).ok()?)
    }
}

#[cfg(test)]
mod tests {
    use num_traits::FromPrimitive;

    use super::FixedBigInt;
    use crate::{Limb, Word};

    #[test]
    fn a_negative_value_is_sign_extended_to_every_limb() {
        let value = FixedBigInt::<8>::from(-2i8);
        assert_eq!(value.as_limbs()[0], Limb::new(Word::MAX - 1));
        assert!(
            value.as_limbs()[1..]
                .iter()
                .all(|limb| *limb == Limb::new(Word::MAX))
        );
    }

    #[test]
    fn an_unsigned_value_keeps_the_limb_above_it_clear() {
        let value = FixedBigInt::<8>::from(u128::MAX);
        assert_eq!(
            value.as_limbs()[(u128::BITS / Word::BITS) as usize],
            Limb::new(0)
        );
    }

    #[test]
    fn from_primitive_accepts_exactly_the_twos_complement_range_of_n_limbs() {
        assert_eq!(
            FixedBigInt::<1>::from_i64(-1).unwrap().as_limbs(),
            [Limb::new(Word::MAX)]
        );
        assert_eq!(
            FixedBigInt::<1>::from_i64(i64::MIN).is_some(),
            Word::BITS == 64
        );
        assert!(FixedBigInt::<1>::from_u64(u64::MAX).is_none());
    }

    #[test]
    fn from_u128_needs_a_spare_bit_for_the_sign() {
        // four limbs are 256 bits on a 64-bit target but only 128 on a 32-bit one
        assert_eq!(
            FixedBigInt::<4>::from_u128(u128::MAX).is_some(),
            Word::BITS == 64
        );
    }

    #[test]
    fn zero_limbs_hold_only_zero() {
        assert!(FixedBigInt::<0>::from_i8(0).is_some());
        assert!(FixedBigInt::<0>::from_i8(-1).is_none());
        assert!(FixedBigInt::<0>::from_u8(1).is_none());
    }
}
