//! Optional initialization-vector parameter abstraction.

use crate::IvParams;

/// Parameters that may provide an initialization vector.
///
/// Key wrappers that define a default for an omitted IV accept this trait
/// instead of [`IvParams`]. Every [`IvParams`] type implements it and returns
/// `Some`; implement it directly on a key-only parameter type to return `None`
/// and select the wrapper's default.
pub trait IvOptParams {
    /// Returns the initialization-vector bytes when supplied, or `None` to
    /// select the key wrapper's default.
    ///
    /// Constant time for every [`IvParams`] type whose `iv` is; other
    /// implementations define their own timing.
    fn iv_opt(&self) -> Option<&[u8]>;
}

impl<T: IvParams + ?Sized> IvOptParams for T {
    /// Returns the [`IvParams::iv`] bytes.
    /// Constant time exactly when that `iv` is.
    fn iv_opt(&self) -> Option<&[u8]> {
        Some(self.iv())
    }
}

#[cfg(test)]
mod tests {
    use super::IvOptParams;
    use crate::KeyWithIvRef;

    struct KeyOnly;

    impl IvOptParams for KeyOnly {
        fn iv_opt(&self) -> Option<&[u8]> {
            None
        }
    }

    #[test]
    fn a_type_with_an_iv_supplies_it_as_the_optional_iv() {
        let iv = [0xa6_u8; 8];
        let params = KeyWithIvRef::new(&[0x42; 16], &iv);
        let params: &dyn IvOptParams = &params;

        assert_eq!(params.iv_opt(), Some(iv.as_slice()));
    }

    #[test]
    fn a_key_only_type_can_omit_the_iv() {
        let params: &dyn IvOptParams = &KeyOnly;

        assert_eq!(params.iv_opt(), None);
    }
}
