//! Addition of [`FixedMontyForm`].

use core::mem;
use core::ops::{Add, AddAssign};

use tc_bigint_v2::{FixedBigUint, LimbArray};

use super::FixedMontyForm;
use crate::monty::add_mod_assign;

/// The sum reduced by the modulus, in place. Panics when the moduli differ, in every build.
/// Constant time, apart from that panic.
impl<const N: usize> AddAssign<&FixedMontyForm<N>> for FixedMontyForm<N> {
    fn add_assign(&mut self, rhs: &FixedMontyForm<N>) {
        self.assert_same_modulus(rhs);
        let mut sum = mem::take(&mut self.value).into_limbs().into_limbs();
        add_mod_assign(
            &mut sum,
            rhs.value.as_limbs(),
            self.params.modulus.as_limbs(),
        );
        self.value = FixedBigUint::new(LimbArray::new(sum));
    }
}

/// The same as `+= &rhs`. Constant time, apart from the panic when the
/// moduli differ.
impl<const N: usize> AddAssign<FixedMontyForm<N>> for FixedMontyForm<N> {
    fn add_assign(&mut self, rhs: FixedMontyForm<N>) {
        *self += &rhs;
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic
/// when the moduli differ.
impl<const N: usize> Add<&FixedMontyForm<N>> for FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn add(mut self, rhs: &FixedMontyForm<N>) -> FixedMontyForm<N> {
        self += rhs;
        self
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic
/// when the moduli differ.
impl<const N: usize> Add<FixedMontyForm<N>> for FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn add(mut self, rhs: FixedMontyForm<N>) -> FixedMontyForm<N> {
        self += &rhs;
        self
    }
}

/// In the storage of `rhs`, as addition is commutative. Constant time,
/// apart from the panic when the moduli differ.
impl<const N: usize> Add<FixedMontyForm<N>> for &FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn add(self, mut rhs: FixedMontyForm<N>) -> FixedMontyForm<N> {
        rhs += self;
        rhs
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic
/// when the moduli differ.
impl<const N: usize> Add<&FixedMontyForm<N>> for &FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn add(self, rhs: &FixedMontyForm<N>) -> FixedMontyForm<N> {
        self.clone() + rhs
    }
}

impl<const N: usize> FixedMontyForm<N> {
    /// Twice `self`, reduced by the modulus. Constant time.
    pub fn double(&self) -> Self {
        let mut doubled = self.clone();
        doubled += self;
        doubled
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int};

    /// `(a + b) mod m`, in `u128`.
    fn add_mod(a: u64, b: u64, m: u64) -> u64 {
        ((u128::from(a % m) + u128::from(b % m)) % u128::from(m)) as u64
    }

    #[test]
    fn the_sum_is_the_primitive_sum_reduced_by_the_modulus() {
        for m in MODULI {
            for a in VALUES {
                for b in VALUES {
                    let sum = (form(a, m) + form(b, m)).retrieve();
                    assert_eq!(sum, int(add_mod(a, b, m)), "{a} {b} {m}");
                }
            }
        }
    }

    #[test]
    fn doubling_is_adding_to_itself() {
        for m in MODULI {
            for a in VALUES {
                assert_eq!(
                    form(a, m).double().retrieve(),
                    int(add_mod(a, a, m)),
                    "{a} {m}"
                );
            }
        }
    }
}
