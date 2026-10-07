//! Forwarding of operators between owned and borrowed operands.
//!
//! Rust needs a separate `impl` for each pairing of owned and borrowed
//! operands. Only the borrowed pairing carries the logic; this macro builds
//! the other five from it, so an operator needs one body instead of six.

/// Builds `T op T`, `T op &T`, `&T op T`, `T op= T` and `T op= &T` from an
/// existing `&T op &T`.
macro_rules! forward_binop {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident,
     [$($generic:tt)*] $ty:ty) => {
        impl<$($generic)*> core::ops::$trait<$ty> for $ty {
            type Output = $ty;

            fn $method(self, rhs: $ty) -> $ty {
                core::ops::$trait::$method(&self, &rhs)
            }
        }

        impl<$($generic)*> core::ops::$trait<&$ty> for $ty {
            type Output = $ty;

            fn $method(self, rhs: &$ty) -> $ty {
                core::ops::$trait::$method(&self, rhs)
            }
        }

        impl<$($generic)*> core::ops::$trait<$ty> for &$ty {
            type Output = $ty;

            fn $method(self, rhs: $ty) -> $ty {
                core::ops::$trait::$method(self, &rhs)
            }
        }

        impl<$($generic)*> core::ops::$assign_trait<$ty> for $ty {
            fn $assign_method(&mut self, rhs: $ty) {
                *self = core::ops::$trait::$method(&*self, &rhs);
            }
        }

        impl<$($generic)*> core::ops::$assign_trait<&$ty> for $ty {
            fn $assign_method(&mut self, rhs: &$ty) {
                *self = core::ops::$trait::$method(&*self, rhs);
            }
        }
    };
}

pub(crate) use forward_binop;
