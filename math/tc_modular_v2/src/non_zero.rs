//! A value known not to be zero.

use core::ops::Deref;

use num_traits::Zero;

/// A value known not to be zero, so that code taking one need not check
/// again. There is no `DerefMut`, which would let the value turn zero.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NonZero<T>(T);

impl<T: Zero> NonZero<T> {
    /// `value` wrapped, or `None` when it is zero. Variable time: only for
    /// public values, as the result shows whether `value` is zero; the check
    /// itself is constant time for the Fixed and Padded types.
    pub fn new(value: T) -> Option<Self> {
        (!value.is_zero()).then_some(Self(value))
    }
}

impl<T> NonZero<T> {
    /// The value, unwrapped. Constant time.
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// The value, borrowed. Constant time.
impl<T> AsRef<T> for NonZero<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

/// The value, borrowed. Constant time.
impl<T> Deref for NonZero<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::NonZero;
    use tc_bigint_v2::{FixedBigInt, FixedBigUint};

    #[test]
    fn only_a_value_other_than_zero_is_wrapped() {
        assert_eq!(NonZero::new(FixedBigUint::<1>::from(0u8)), None);
        assert_eq!(NonZero::new(FixedBigUint::<0>::default()), None);
        let seven = FixedBigUint::<1>::from(7u8);
        let wrapped = NonZero::new(seven.clone()).unwrap();
        assert_eq!(wrapped.as_ref(), &seven);
        assert_eq!(*wrapped, seven);
        assert_eq!(wrapped.into_inner(), seven);
        assert!(NonZero::new(FixedBigInt::<1>::from(-1i8)).is_some());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn the_heap_types_are_wrapped_at_their_width() {
        use tc_bigint_v2::{BigUint, PaddedBigUint};

        assert_eq!(NonZero::new(PaddedBigUint::from(0u128)), None);
        let wrapped = NonZero::new(PaddedBigUint::from(7u128)).unwrap();
        assert_eq!(
            wrapped.as_limbs().len(),
            PaddedBigUint::from(7u128).as_limbs().len()
        );
        assert_eq!(NonZero::new(BigUint::default()), None);
        assert!(NonZero::new(BigUint::from(7u8)).is_some());
    }
}
