//! Addition of [`BigMontyForm`].

use core::ops::{Add, AddAssign};

use super::BigMontyForm;
use crate::monty::add_mod_assign;

/// The sum reduced by the modulus, in place. Panics when the moduli differ, in every build.
/// Constant time, apart from that panic.
impl<'a> AddAssign<&BigMontyForm<'a>> for BigMontyForm<'a> {
    fn add_assign(&mut self, rhs: &BigMontyForm<'a>) {
        self.assert_same_modulus(rhs);
        add_mod_assign(&mut self.value, &rhs.value, self.params.modulus.as_limbs());
    }
}

/// The same as `+= &rhs`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> AddAssign<BigMontyForm<'a>> for BigMontyForm<'a> {
    fn add_assign(&mut self, rhs: BigMontyForm<'a>) {
        *self += &rhs;
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Add<&BigMontyForm<'a>> for BigMontyForm<'a> {
    type Output = BigMontyForm<'a>;

    fn add(mut self, rhs: &BigMontyForm<'a>) -> BigMontyForm<'a> {
        self += rhs;
        self
    }
}

/// In the storage of `self`, as `+=`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Add<BigMontyForm<'a>> for BigMontyForm<'a> {
    type Output = BigMontyForm<'a>;

    fn add(mut self, rhs: BigMontyForm<'a>) -> BigMontyForm<'a> {
        self += &rhs;
        self
    }
}

/// In the storage of `rhs`, as addition is commutative. Constant time,
/// apart from the panic when the moduli differ.
impl<'a> Add<BigMontyForm<'a>> for &BigMontyForm<'a> {
    type Output = BigMontyForm<'a>;

    fn add(self, mut rhs: BigMontyForm<'a>) -> BigMontyForm<'a> {
        rhs += self;
        rhs
    }
}

/// In a copy of `self`, in a new buffer. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Add<&BigMontyForm<'a>> for &BigMontyForm<'a> {
    type Output = BigMontyForm<'a>;

    fn add(self, rhs: &BigMontyForm<'a>) -> BigMontyForm<'a> {
        self.clone() + rhs
    }
}

impl BigMontyForm<'_> {
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
