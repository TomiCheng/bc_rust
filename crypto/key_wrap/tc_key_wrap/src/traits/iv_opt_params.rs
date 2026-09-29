//! Optional initialization-vector parameter abstraction.

use crate::IvParams;
use tc_block_cipher::{KeyFixed, KeyOwned, KeyRef};

/// Parameters that may provide an initialization vector.
///
/// Key wrappers that define a default for an omitted IV accept this trait
/// instead of [`IvParams`]. Every [`IvParams`] type implements it and returns
/// `Some`. The key-only containers of `tc_block_cipher`, [`KeyRef`],
/// [`KeyFixed`] and [`KeyOwned`], implement it and return `None`, selecting the
/// wrapper's default; implement it the same way on your own key-only parameter
/// type.
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

impl IvOptParams for KeyRef<'_> {
    /// Returns `None`: a key alone selects the wrapper's default IV.
    /// Constant time.
    fn iv_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl<const N: usize> IvOptParams for KeyFixed<N> {
    /// Returns `None`: a key alone selects the wrapper's default IV.
    /// Constant time.
    fn iv_opt(&self) -> Option<&[u8]> {
        None
    }
}

impl IvOptParams for KeyOwned {
    /// Returns `None`: a key alone selects the wrapper's default IV.
    /// Constant time.
    fn iv_opt(&self) -> Option<&[u8]> {
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec;

    use super::IvOptParams;
    use crate::KeyWithIvRef;
    use tc_block_cipher::{KeyFixed, KeyOwned, KeyRef};

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

    #[test]
    fn the_block_cipher_key_containers_omit_the_iv() {
        let key = [0x42_u8; 16];

        assert_eq!(KeyRef::new(&key).iv_opt(), None);
        assert_eq!(KeyFixed::new(key).iv_opt(), None);
        assert_eq!(KeyOwned::new(vec![0x42; 16]).iv_opt(), None);
    }
}
