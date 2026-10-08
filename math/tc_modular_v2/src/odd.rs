//! A value known to be odd.

use core::ops::Deref;

use tc_bigint_v2::BitOps;

/// A value known to be odd, as a Montgomery modulus must be, so that code
/// taking one need not check again; an odd value is never zero. There is
/// no `DerefMut`, which would let the value turn even.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Odd<T>(T);

impl<T: BitOps> Odd<T> {
    /// `value` wrapped, or `None` when it is even. Variable time: only for
    /// public values, as the result shows whether `value` is odd; reading
    /// the bit itself is constant time for the Fixed and Padded types.
    pub fn new(value: T) -> Option<Self> {
        value.bit(0).then_some(Self(value))
    }
}

impl<T> Odd<T> {
    /// The value, unwrapped. Constant time.
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// The value, borrowed. Constant time.
impl<T> AsRef<T> for Odd<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

/// The value, borrowed. Constant time.
impl<T> Deref for Odd<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Odd;
    use tc_bigint_v2::{FixedBigInt, FixedBigUint};

    #[test]
    fn only_an_odd_value_is_wrapped() {
        assert_eq!(Odd::new(FixedBigUint::<1>::from(0u8)), None);
        assert_eq!(Odd::new(FixedBigUint::<1>::from(6u8)), None);
        assert_eq!(Odd::new(FixedBigUint::<0>::default()), None);
        let seven = FixedBigUint::<1>::from(7u8);
        let wrapped = Odd::new(seven.clone()).unwrap();
        assert_eq!(wrapped.as_ref(), &seven);
        assert_eq!(*wrapped, seven);
        assert_eq!(wrapped.into_inner(), seven);
    }

    #[test]
    fn a_negative_value_is_odd_by_its_low_bit() {
        assert!(Odd::new(FixedBigInt::<1>::from(-3i8)).is_some());
        assert_eq!(Odd::new(FixedBigInt::<1>::from(-4i8)), None);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn the_heap_types_are_wrapped_too() {
        use tc_bigint_v2::{BigInt, BigUint, PaddedBigUint};

        assert!(Odd::new(PaddedBigUint::from(7u128)).is_some());
        assert_eq!(Odd::new(BigUint::default()), None);
        assert!(Odd::new(BigUint::from(7u8)).is_some());
        assert!(Odd::new(BigInt::from(-7i8)).is_some());
    }
}
