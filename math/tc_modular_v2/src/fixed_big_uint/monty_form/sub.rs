//! Subtraction of [`FixedMontyForm`].

use core::mem;
use core::ops::{Sub, SubAssign};

use tc_bigint_v2::{FixedBigUint, LimbArray};

use super::FixedMontyForm;
use crate::monty::sub_mod_assign;

/// The difference reduced by the modulus, never negative, in place. Panics when the moduli differ, in every build.
/// Constant time, apart from that panic.
impl<const N: usize> SubAssign<&FixedMontyForm<N>> for FixedMontyForm<N> {
    fn sub_assign(&mut self, rhs: &FixedMontyForm<N>) {
        self.assert_same_modulus(rhs);
        let mut difference = mem::take(&mut self.value).into_limbs().into_limbs();
        sub_mod_assign(
            &mut difference,
            rhs.value.as_limbs(),
            self.params.modulus.as_limbs(),
        );
        self.value = FixedBigUint::new(LimbArray::new(difference));
    }
}

/// The same as `-= &rhs`. Constant time, apart from the panic when the
/// moduli differ.
impl<const N: usize> SubAssign<FixedMontyForm<N>> for FixedMontyForm<N> {
    fn sub_assign(&mut self, rhs: FixedMontyForm<N>) {
        *self -= &rhs;
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// when the moduli differ.
impl<const N: usize> Sub<&FixedMontyForm<N>> for FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn sub(mut self, rhs: &FixedMontyForm<N>) -> FixedMontyForm<N> {
        self -= rhs;
        self
    }
}

/// In the storage of `self`, as `-=`. Constant time, apart from the panic
/// when the moduli differ.
impl<const N: usize> Sub<FixedMontyForm<N>> for FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn sub(mut self, rhs: FixedMontyForm<N>) -> FixedMontyForm<N> {
        self -= &rhs;
        self
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic
/// when the moduli differ.
impl<const N: usize> Sub<FixedMontyForm<N>> for &FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn sub(self, rhs: FixedMontyForm<N>) -> FixedMontyForm<N> {
        self.clone() - &rhs
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic
/// when the moduli differ.
impl<const N: usize> Sub<&FixedMontyForm<N>> for &FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn sub(self, rhs: &FixedMontyForm<N>) -> FixedMontyForm<N> {
        self.clone() - rhs
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int};

    /// `(a - b) mod m`, never negative, in `u128`.
    fn sub_mod(a: u64, b: u64, m: u64) -> u64 {
        ((u128::from(a % m) + u128::from(m) - u128::from(b % m)) % u128::from(m)) as u64
    }

    #[test]
    fn the_difference_is_the_primitive_difference_reduced_by_the_modulus() {
        for m in MODULI {
            for a in VALUES {
                for b in VALUES {
                    let difference = (form(a, m) - form(b, m)).retrieve();
                    assert_eq!(difference, int(sub_mod(a, b, m)), "{a} {b} {m}");
                }
            }
        }
    }
}
