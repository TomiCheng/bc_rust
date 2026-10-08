//! Support for the operators.
//!
//! Each operator keeps its logic in its in-place form, and its other forms
//! are written around that, reusing the storage of an operand passed by
//! value; only a form whose operands are all borrowed works in a copy,
//! which `CloneFor` makes for the heap types.

/// A copy of a value with room for an operation with `other`, made in one
/// allocation, so that the in-place operator applied to it next does not
/// have to grow it again.
pub(crate) trait CloneFor {
    /// The copy, already as wide as the operation with `other` needs.
    fn clone_for(&self, other: &Self) -> Self;
}
