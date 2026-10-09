//! Addition of [`PaddedMontyForm`].

use core::mem;
use core::ops::{Add, AddAssign};

use tc_bigint_v2::PaddedBigUint;

use super::PaddedMontyForm;
use crate::monty::add_mod_assign;

/// The sum reduced by the modulus, in place. Panics when the moduli differ, in every build.
/// Constant time, apart from that panic.
impl<'a> AddAssign<&PaddedMontyForm<'a>> for PaddedMontyForm<'a> {
    fn add_assign(&mut self, rhs: &PaddedMontyForm<'a>) {
        self.assert_same_modulus(rhs);
        let mut sum = mem::take(&mut self.value).into_limbs();
        add_mod_assign(
            &mut sum,
            rhs.value.as_limbs(),
            self.params.modulus.as_limbs(),
        );
        self.value = PaddedBigUint::new(sum);
    }
}

/// The same as `+= &rhs`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> AddAssign<PaddedMontyForm<'a>> for PaddedMontyForm<'a> {
    fn add_assign(&mut self, rhs: PaddedMontyForm<'a>) {
        *self += &rhs;
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Add<&PaddedMontyForm<'a>> for PaddedMontyForm<'a> {
    type Output = PaddedMontyForm<'a>;

    fn add(mut self, rhs: &PaddedMontyForm<'a>) -> PaddedMontyForm<'a> {
        self += rhs;
        self
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Add<PaddedMontyForm<'a>> for PaddedMontyForm<'a> {
    type Output = PaddedMontyForm<'a>;

    fn add(mut self, rhs: PaddedMontyForm<'a>) -> PaddedMontyForm<'a> {
        self += &rhs;
        self
    }
}

/// In the storage of `rhs`, as addition is commutative. Constant time,
/// apart from the panic when the moduli differ.
impl<'a> Add<PaddedMontyForm<'a>> for &PaddedMontyForm<'a> {
    type Output = PaddedMontyForm<'a>;

    fn add(self, mut rhs: PaddedMontyForm<'a>) -> PaddedMontyForm<'a> {
        rhs += self;
        rhs
    }
}

/// In a copy of `self`, in a new buffer. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Add<&PaddedMontyForm<'a>> for &PaddedMontyForm<'a> {
    type Output = PaddedMontyForm<'a>;

    fn add(self, rhs: &PaddedMontyForm<'a>) -> PaddedMontyForm<'a> {
        self.clone() + rhs
    }
}

impl PaddedMontyForm<'_> {
    /// Twice `self`, reduced by the modulus. Constant time.
    pub fn double(&self) -> Self {
        let mut doubled = self.clone();
        doubled += self;
        doubled
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int, params};

    /// `(a + b) mod m`, in `u128`.
    fn add_mod(a: u64, b: u64, m: u64) -> u64 {
        ((u128::from(a % m) + u128::from(b % m)) % u128::from(m)) as u64
    }

    #[test]
    fn the_sum_is_the_primitive_sum_reduced_by_the_modulus() {
        for m in MODULI {
            let params = params(m);
            for a in VALUES {
                for b in VALUES {
                    let sum = (form(a, &params) + form(b, &params)).retrieve();
                    assert_eq!(sum, int(add_mod(a, b, m)), "{a} {b} {m}");
                }
            }
        }
    }

    #[test]
    fn doubling_is_adding_to_itself() {
        for m in MODULI {
            let params = params(m);
            for a in VALUES {
                let doubled = form(a, &params).double().retrieve();
                assert_eq!(doubled, int(add_mod(a, a, m)), "{a} {m}");
            }
        }
    }
}
