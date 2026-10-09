//! The declaration of [`BigMontyForm`], and the way in and out of
//! Montgomery form.

use alloc::boxed::Box;
use alloc::vec;

use tc_bigint_v2::{BigUint, Limb};

use crate::BigMontyParams;
use crate::monty::monty_mul;

/// A value modulo the odd modulus of the [`BigMontyParams`] it borrows, kept
/// as `x · R mod m` in as many limbs as the modulus takes, as a `BigUint`
/// would trim them. Going in and out of the form is variable time, as for
/// `BigUint`; the arithmetic in between works on those limbs in constant
/// time. Operations on two forms need the same modulus, and panic
/// otherwise, in every build.
#[derive(Clone, Debug)]
pub struct BigMontyForm<'a> {
    /// `x · R mod m`, in as many limbs as the modulus takes.
    pub(super) value: Box<[Limb]>,
    pub(super) params: &'a BigMontyParams,
}

impl<'a> BigMontyForm<'a> {
    /// `value mod m` in Montgomery form: reduced, then multiplied by
    /// `R² mod m`. Variable time: only for public values; secret ones go
    /// through [`PaddedMontyForm`].
    ///
    /// [`PaddedMontyForm`]: crate::PaddedMontyForm
    pub fn new(value: &BigUint, params: &'a BigMontyParams) -> Self {
        let reduced = padded(&(value % &params.modulus), params);
        let value = product(&reduced, &padded(&params.r2, params), params);
        Self { value, params }
    }

    /// Zero, which is zero in Montgomery form too. Constant time.
    pub fn zero(params: &'a BigMontyParams) -> Self {
        let value = vec![Limb::new(0); params.modulus.as_limbs().len()].into_boxed_slice();
        Self { value, params }
    }

    /// One, which is `R mod m` in Montgomery form. Variable time: only for
    /// public parameters; secret ones go through [`PaddedMontyForm`].
    ///
    /// [`PaddedMontyForm`]: crate::PaddedMontyForm
    pub fn one(params: &'a BigMontyParams) -> Self {
        Self {
            value: padded(&params.one, params),
            params,
        }
    }

    /// The value out of Montgomery form, below the modulus: a Montgomery
    /// multiplication by one takes the factor `R` away. Variable time, as
    /// the `BigUint` it gives back is trimmed: only for public values;
    /// secret ones go through [`PaddedMontyForm`].
    ///
    /// [`PaddedMontyForm`]: crate::PaddedMontyForm
    pub fn retrieve(&self) -> BigUint {
        let mut one = vec![Limb::new(0); self.value.len()];
        one[0] = Limb::new(1);
        BigUint::new(product(&self.value, &one, self.params).into_vec())
    }

    /// The parameters. Constant time.
    pub fn params(&self) -> &'a BigMontyParams {
        self.params
    }

    /// Panics when `rhs` has another modulus, in every build. Variable time:
    /// the moduli are public.
    pub(super) fn assert_same_modulus(&self, rhs: &Self) {
        assert!(
            self.params.modulus == rhs.params.modulus,
            "operands of different moduli"
        );
    }
}

/// `value`, below the modulus, in as many limbs as the modulus takes.
/// Variable time: the limbs `value` takes show.
fn padded(value: &BigUint, params: &BigMontyParams) -> Box<[Limb]> {
    let mut limbs = value.as_limbs().to_vec();
    limbs.resize(params.modulus.as_limbs().len(), Limb::new(0));
    limbs.into_boxed_slice()
}

/// `lhs · rhs · R⁻¹ mod m`, by one Montgomery multiplication into a new
/// buffer. Constant time.
pub(super) fn product(lhs: &[Limb], rhs: &[Limb], params: &BigMontyParams) -> Box<[Limb]> {
    let mut output = vec![Limb::new(0); lhs.len()].into_boxed_slice();
    monty_mul(
        &mut output,
        lhs,
        rhs,
        params.modulus.as_limbs(),
        params.inverse,
    );
    output
}

#[cfg(test)]
mod tests {
    use super::super::testing::{MODULI, VALUES, form, int, params};
    use super::BigMontyForm;

    #[test]
    fn a_value_comes_back_out_reduced_by_the_modulus() {
        for m in MODULI {
            let params = params(m);
            for a in VALUES {
                assert_eq!(form(a, &params).retrieve(), int(a % m), "{a} {m}");
            }
        }
    }

    #[test]
    fn zero_and_one_come_back_out_as_themselves_reduced() {
        for m in MODULI {
            let params = params(m);
            assert_eq!(BigMontyForm::zero(&params).retrieve(), int(0), "{m}");
            assert_eq!(BigMontyForm::one(&params).retrieve(), int(1 % m), "{m}");
        }
    }

    #[test]
    #[should_panic(expected = "operands of different moduli")]
    fn operands_of_different_moduli_panic() {
        let (three, five) = (params(3), params(5));
        let _ = form(1, &three) * form(1, &five);
    }
}
