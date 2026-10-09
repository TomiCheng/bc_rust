//! Multiplication of [`BigMontyForm`].

use core::ops::{Mul, MulAssign};

use super::BigMontyForm;
use super::types::product;

/// One Montgomery multiplication, into a new buffer. Panics when the moduli differ, in every build.
/// Constant time, apart from that panic.
impl<'a> MulAssign<&BigMontyForm<'a>> for BigMontyForm<'a> {
    fn mul_assign(&mut self, rhs: &BigMontyForm<'a>) {
        self.assert_same_modulus(rhs);
        self.value = product(&self.value, &rhs.value, self.params);
    }
}

/// The same as `*= &rhs`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> MulAssign<BigMontyForm<'a>> for BigMontyForm<'a> {
    fn mul_assign(&mut self, rhs: BigMontyForm<'a>) {
        *self *= &rhs;
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Mul<&BigMontyForm<'a>> for BigMontyForm<'a> {
    type Output = BigMontyForm<'a>;

    fn mul(mut self, rhs: &BigMontyForm<'a>) -> BigMontyForm<'a> {
        self *= rhs;
        self
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Mul<BigMontyForm<'a>> for BigMontyForm<'a> {
    type Output = BigMontyForm<'a>;

    fn mul(mut self, rhs: BigMontyForm<'a>) -> BigMontyForm<'a> {
        self *= &rhs;
        self
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Constant time,
/// apart from the panic when the moduli differ.
impl<'a> Mul<BigMontyForm<'a>> for &BigMontyForm<'a> {
    type Output = BigMontyForm<'a>;

    fn mul(self, mut rhs: BigMontyForm<'a>) -> BigMontyForm<'a> {
        rhs *= self;
        rhs
    }
}

/// In a copy of `self`, in a new buffer. Constant time, apart from the panic when the
/// moduli differ.
impl<'a> Mul<&BigMontyForm<'a>> for &BigMontyForm<'a> {
    type Output = BigMontyForm<'a>;

    fn mul(self, rhs: &BigMontyForm<'a>) -> BigMontyForm<'a> {
        self.clone() * rhs
    }
}

impl BigMontyForm<'_> {
    /// `self` times itself, by one Montgomery multiplication. Constant time.
    pub fn square(&self) -> Self {
        self * self
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int, mul_mod, params};

    #[test]
    fn the_product_is_the_primitive_product_reduced_by_the_modulus() {
        for m in MODULI {
            let params = params(m);
            for a in VALUES {
                for b in VALUES {
                    let product = (form(a, &params) * form(b, &params)).retrieve();
                    assert_eq!(product, int(mul_mod(a % m, b % m, m)), "{a} {b} {m}");
                }
            }
        }
    }

    #[test]
    fn squaring_is_multiplying_by_itself() {
        for m in MODULI {
            let params = params(m);
            for a in VALUES {
                let squared = form(a, &params).square().retrieve();
                assert_eq!(squared, int(mul_mod(a % m, a % m, m)), "{a} {m}");
            }
        }
    }
}
