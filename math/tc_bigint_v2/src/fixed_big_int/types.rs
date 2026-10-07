use crate::LimbArray;

/// Signed integer of `N` limbs.
pub struct FixedBigInt<const N: usize> {
    limbs: LimbArray<N>,
}
