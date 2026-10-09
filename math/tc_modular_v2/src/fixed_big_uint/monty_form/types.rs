//! The declaration of [`FixedMontyForm`], and the way in and out of
//! Montgomery form.

use num_traits::{One, Zero};
use tc_bigint_v2::{FixedBigUint, Limb, LimbArray};

use crate::FixedMontyParams;
use crate::monty::monty_mul;
use tc_zeroize::Zeroizing;

/// A value modulo the odd modulus of its [`FixedMontyParams`], kept as
/// `x · R mod m`, so that a product takes one Montgomery multiplication
/// rather than a division. The form holds its own copy of the parameters,
/// on the stack. Operations on two forms need the same modulus, and panic
/// otherwise, in every build.
#[derive(Clone, Debug)]
pub struct FixedMontyForm<const N: usize> {
    /// `x · R mod m`.
    pub(super) value: FixedBigUint<N>,
    pub(super) params: FixedMontyParams<N>,
}

impl<const N: usize> FixedMontyForm<N> {
    /// `value mod m` in Montgomery form: reduced, then multiplied by
    /// `R² mod m`. Constant time.
    pub fn new(value: &FixedBigUint<N>, params: FixedMontyParams<N>) -> Self {
        // The residue is wiped once it is in the form.
        let reduced = Zeroizing::new(value % &params.modulus);
        let value = product(&reduced, &params.r2, &params);
        Self { value, params }
    }

    /// Zero, which is zero in Montgomery form too. Constant time.
    pub fn zero(params: FixedMontyParams<N>) -> Self {
        Self {
            value: FixedBigUint::zero(),
            params,
        }
    }

    /// One, which is `R mod m` in Montgomery form. Constant time.
    pub fn one(params: FixedMontyParams<N>) -> Self {
        Self {
            value: params.one.clone(),
            params,
        }
    }

    /// The value out of Montgomery form, below the modulus: a Montgomery
    /// multiplication by one takes the factor `R` away. Constant time.
    pub fn retrieve(&self) -> FixedBigUint<N> {
        product(&self.value, &FixedBigUint::one(), &self.params)
    }

    /// The parameters. Constant time.
    pub fn params(&self) -> &FixedMontyParams<N> {
        &self.params
    }

    /// Panics when `rhs` has another modulus, in every build. Constant time
    /// apart from that panic: the moduli are compared in constant time.
    pub(super) fn assert_same_modulus(&self, rhs: &Self) {
        assert!(
            self.params.modulus == rhs.params.modulus,
            "operands of different moduli"
        );
    }
}

/// `lhs · rhs · R⁻¹ mod m`, by one Montgomery multiplication into a new
/// array on the stack. Constant time.
pub(super) fn product<const N: usize>(
    lhs: &FixedBigUint<N>,
    rhs: &FixedBigUint<N>,
    params: &FixedMontyParams<N>,
) -> FixedBigUint<N> {
    let mut output = [Limb::new(0); N];
    monty_mul(
        &mut output,
        lhs.as_limbs(),
        rhs.as_limbs(),
        params.modulus.as_limbs(),
        params.inverse,
    );
    FixedBigUint::new(LimbArray::new(output))
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int, params};
    use super::FixedMontyForm;

    #[test]
    fn a_value_comes_back_out_reduced_by_the_modulus() {
        for m in MODULI {
            for a in VALUES {
                assert_eq!(form(a, m).retrieve(), int(a % m), "{a} {m}");
            }
        }
    }

    #[test]
    fn zero_and_one_come_back_out_as_themselves_reduced() {
        for m in MODULI {
            assert_eq!(FixedMontyForm::zero(params(m)).retrieve(), int(0), "{m}");
            assert_eq!(FixedMontyForm::one(params(m)).retrieve(), int(1 % m), "{m}");
        }
    }

    #[test]
    #[should_panic(expected = "operands of different moduli")]
    fn operands_of_different_moduli_panic() {
        let _ = form(1, 3) * form(1, 5);
    }
}
