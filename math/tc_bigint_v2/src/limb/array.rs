//! A fixed number of limbs.

use core::fmt;
use core::hash::{Hash, Hasher};

use core::cmp::Ordering;

use tc_constant_time::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeOrd};
use tc_zeroize::Zeroize;

use super::{Limb, ct_eq_extended, ct_lt_extended};

/// `N` limbs, least significant first.
#[derive(Clone)]
pub struct LimbArray<const N: usize>([Limb; N]);

/// Every limb zero. Written out because the standard library implements
/// `Default` only for arrays of up to 32 elements. Constant time.
impl<const N: usize> Default for LimbArray<N> {
    fn default() -> Self {
        Self([Limb::new(0); N])
    }
}

/// Prints only the type and its width, never the value, so a secret cannot
/// reach a log or a panic message. Constant time.
impl<const N: usize> fmt::Debug for LimbArray<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LimbArray")
            .field("limbs", &N)
            .finish_non_exhaustive()
    }
}

impl<const N: usize> LimbArray<N> {
    /// Wraps `N` limbs, least significant first. Constant time.
    pub const fn new(limbs: [Limb; N]) -> Self {
        Self(limbs)
    }

    /// The limbs, least significant first. Constant time.
    pub const fn as_slice(&self) -> &[Limb] {
        &self.0
    }

    /// Unwraps the limbs, least significant first. Constant time.
    pub const fn into_limbs(self) -> [Limb; N] {
        self.0
    }

    /// The limbs for in-place arithmetic inside the crate.
    pub(crate) fn as_mut_slice(&mut self) -> &mut [Limb] {
        &mut self.0
    }
}

/// Constant time.
impl<const N: usize> ConstantTimeEq for LimbArray<N> {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        ct_eq_extended(&self.0, 0, &rhs.0, 0)
    }
}

/// Goes through [`ConstantTimeEq::ct_eq`], so `==` is constant time.
impl<const N: usize> PartialEq for LimbArray<N> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl<const N: usize> Eq for LimbArray<N> {}

/// Hashes every limb, which agrees with `==` as both sides have `N` limbs.
impl<const N: usize> Hash for LimbArray<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Constant time.
impl<const N: usize> ConstantTimeOrd for LimbArray<N> {
    fn ct_lt(&self, rhs: &Self) -> Choice {
        ct_lt_extended(&self.0, 0, &rhs.0, 0, false)
    }
}

/// Goes through [`ConstantTimeOrd::ct_lt`] and `ct_eq`, so it is constant time.
impl<const N: usize> PartialOrd for LimbArray<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Goes through [`ConstantTimeOrd::ct_lt`] and `ct_eq`, so it is constant time.
impl<const N: usize> Ord for LimbArray<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        let less = self.ct_lt(other).unwrap_u8() == 1;
        let equal = self.ct_eq(other).unwrap_u8() == 1;
        match (less, equal) {
            (true, _) => Ordering::Less,
            (false, true) => Ordering::Equal,
            (false, false) => Ordering::Greater,
        }
    }
}

/// Overwrites every limb with zero through volatile writes. Constant time.
impl<const N: usize> Zeroize for LimbArray<N> {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

/// The same choice for every limb; assigning and swapping work in place.
/// Constant time.
impl<const N: usize> ConditionallySelectable for LimbArray<N> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(<[Limb; N]>::conditional_select(&a.0, &b.0, choice))
    }

    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        self.0.conditional_assign(&other.0, choice);
    }

    fn conditional_swap(a: &mut Self, b: &mut Self, choice: Choice) {
        <[Limb; N]>::conditional_swap(&mut a.0, &mut b.0, choice);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn zeroizing_leaves_every_limb_zero() {
        use tc_zeroize::Zeroize;

        let mut array = super::LimbArray::new([super::Limb::new(7); 3]);
        array.zeroize();
        assert_eq!(array.as_slice(), [super::Limb::new(0); 3]);
    }

    #[test]
    fn selecting_takes_one_array_whole() {
        use tc_constant_time::{Choice, ConditionallySelectable};

        use crate::{Limb, LimbArray};

        let (a, b) = (
            LimbArray::new([Limb::new(1); 2]),
            LimbArray::new([Limb::new(2); 2]),
        );
        let picked = |bit| LimbArray::conditional_select(&a, &b, Choice::from_lsb(bit));
        assert_eq!((picked(0), picked(1)), (a, b));
    }
}
