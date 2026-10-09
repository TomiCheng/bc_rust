//! Multiplication of [`FixedMontyForm`].

use core::ops::{Mul, MulAssign};

use super::FixedMontyForm;
use super::types::product;

/// One Montgomery multiplication, into a new array on the stack. Panics when the moduli differ, in every build.
/// Constant time, apart from that panic.
impl<const N: usize> MulAssign<&FixedMontyForm<N>> for FixedMontyForm<N> {
    fn mul_assign(&mut self, rhs: &FixedMontyForm<N>) {
        self.assert_same_modulus(rhs);
        self.value = product(&self.value, &rhs.value, &self.params);
    }
}

/// The same as `*= &rhs`. Constant time, apart from the panic when the
/// moduli differ.
impl<const N: usize> MulAssign<FixedMontyForm<N>> for FixedMontyForm<N> {
    fn mul_assign(&mut self, rhs: FixedMontyForm<N>) {
        *self *= &rhs;
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic
/// when the moduli differ.
impl<const N: usize> Mul<&FixedMontyForm<N>> for FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn mul(mut self, rhs: &FixedMontyForm<N>) -> FixedMontyForm<N> {
        self *= rhs;
        self
    }
}

/// In the storage of `self`, as `*=`. Constant time, apart from the panic
/// when the moduli differ.
impl<const N: usize> Mul<FixedMontyForm<N>> for FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn mul(mut self, rhs: FixedMontyForm<N>) -> FixedMontyForm<N> {
        self *= &rhs;
        self
    }
}

/// In the storage of `rhs`, as multiplication is commutative. Constant time,
/// apart from the panic when the moduli differ.
impl<const N: usize> Mul<FixedMontyForm<N>> for &FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn mul(self, mut rhs: FixedMontyForm<N>) -> FixedMontyForm<N> {
        rhs *= self;
        rhs
    }
}

/// In a copy of `self`, on the stack. Constant time, apart from the panic
/// when the moduli differ.
impl<const N: usize> Mul<&FixedMontyForm<N>> for &FixedMontyForm<N> {
    type Output = FixedMontyForm<N>;

    fn mul(self, rhs: &FixedMontyForm<N>) -> FixedMontyForm<N> {
        self.clone() * rhs
    }
}

impl<const N: usize> FixedMontyForm<N> {
    /// `self` times itself, by one Montgomery multiplication. Constant time.
    pub fn square(&self) -> Self {
        self * self
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int, mul_mod};

    #[test]
    fn the_product_is_the_primitive_product_reduced_by_the_modulus() {
        for m in MODULI {
            for a in VALUES {
                for b in VALUES {
                    let product = (form(a, m) * form(b, m)).retrieve();
                    assert_eq!(product, int(mul_mod(a % m, b % m, m)), "{a} {b} {m}");
                }
            }
        }
    }

    #[test]
    fn squaring_is_multiplying_by_itself() {
        for m in MODULI {
            for a in VALUES {
                assert_eq!(
                    form(a, m).square().retrieve(),
                    int(mul_mod(a % m, a % m, m)),
                    "{a} {m}"
                );
            }
        }
    }
}
