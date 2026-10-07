use crate::LimbArray;

/// Unsigned integer of `N` limbs.
pub struct FixedBigUint<const N: usize> {
    limbs: LimbArray<N>,
}
