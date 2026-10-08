//! Forwarding of operators between owned and borrowed operands.
//!
//! Rust needs a separate `impl` for each pairing of owned and borrowed
//! operands. The logic lives in the in-place `T op= &T`; the macros build
//! the other forms from it, reusing the storage of an operand that is
//! passed by value, so only `&T op &T` has to allocate, and that once.

/// A copy of a value with room for an operation with `other`, made in one
/// allocation, so that the in-place operator applied to it next does not
/// have to grow it again.
pub(crate) trait CloneFor {
    /// The copy, already as wide as the operation with `other` needs.
    fn clone_for(&self, other: &Self) -> Self;
}

/// Builds `T op T`, `T op &T`, `&T op T`, `&T op &T` and `T op= T` from an
/// existing `T op= &T`, for an operator where `a op b` equals `b op a`:
/// `&T op T` then works in the storage of its right operand.
macro_rules! forward_commutative_binop {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident,
     [$($generic:tt)*] $ty:ty) => {
        impl<$($generic)*> core::ops::$trait<$ty> for $ty {
            type Output = $ty;

            fn $method(mut self, rhs: $ty) -> $ty {
                core::ops::$assign_trait::$assign_method(&mut self, &rhs);
                self
            }
        }

        impl<$($generic)*> core::ops::$trait<&$ty> for $ty {
            type Output = $ty;

            fn $method(mut self, rhs: &$ty) -> $ty {
                core::ops::$assign_trait::$assign_method(&mut self, rhs);
                self
            }
        }

        impl<$($generic)*> core::ops::$trait<$ty> for &$ty {
            type Output = $ty;

            fn $method(self, mut rhs: $ty) -> $ty {
                core::ops::$assign_trait::$assign_method(&mut rhs, self);
                rhs
            }
        }

        impl<$($generic)*> core::ops::$trait<&$ty> for &$ty {
            type Output = $ty;

            fn $method(self, rhs: &$ty) -> $ty {
                let mut result = crate::ops::CloneFor::clone_for(self, rhs);
                core::ops::$assign_trait::$assign_method(&mut result, rhs);
                result
            }
        }

        impl<$($generic)*> core::ops::$assign_trait<$ty> for $ty {
            fn $assign_method(&mut self, rhs: $ty) {
                core::ops::$assign_trait::$assign_method(self, &rhs);
            }
        }
    };
}

/// Builds `op &T` from an existing `op T`, which works in place in its
/// operand: the borrowed form clones first, so only it allocates.
macro_rules! forward_unop {
    ($trait:ident, $method:ident, [$($generic:tt)*] $ty:ty) => {
        impl<$($generic)*> core::ops::$trait for &$ty {
            type Output = $ty;

            fn $method(self) -> $ty {
                core::ops::$trait::$method(self.clone())
            }
        }
    };
}

pub(crate) use {forward_commutative_binop, forward_unop};
