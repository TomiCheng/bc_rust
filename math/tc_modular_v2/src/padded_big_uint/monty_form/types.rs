//! The declaration of [`PaddedMontyForm`], and the way in and out of
//! Montgomery form.

use alloc::vec;

use num_traits::{One, Zero};
use tc_bigint_v2::{Limb, PaddedBigUint};

use crate::PaddedMontyParams;
use crate::monty::monty_mul;
use tc_zeroize::Zeroizing;

/// A value modulo the odd modulus of the [`PaddedMontyParams`] it borrows,
/// kept as `x · R mod m` at the width of the modulus, so that a product
/// takes one Montgomery multiplication rather than a division. Operations
/// on two forms need the same modulus at the same width, and panic
/// otherwise, in every build.
#[derive(Clone, Debug)]
pub struct PaddedMontyForm<'a> {
    /// `x · R mod m`, at the width of the modulus.
    pub(super) value: PaddedBigUint,
    pub(super) params: &'a PaddedMontyParams,
}

impl<'a> PaddedMontyForm<'a> {
    /// `value mod m` in Montgomery form: reduced, then multiplied by
    /// `R² mod m`, at the width of the modulus whatever the width of
    /// `value`. Constant time: the widths are public.
    pub fn new(value: &PaddedBigUint, params: &'a PaddedMontyParams) -> Self {
        // The residue is wiped once it is in the form.
        let reduced = Zeroizing::new(value % &params.modulus);
        // The residue fits the width of the modulus, which `%` reaches or
        // exceeds.
        let low = &reduced.as_limbs()[..params.modulus.as_limbs().len()];
        let value = product(low, params.r2.as_limbs(), params);
        Self { value, params }
    }

    /// Zero, which is zero in Montgomery form too, at the width of the
    /// modulus. Constant time.
    pub fn zero(params: &'a PaddedMontyParams) -> Self {
        let mut value = params.modulus.clone();
        value.set_zero();
        Self { value, params }
    }

    /// One, which is `R mod m` in Montgomery form. Constant time.
    pub fn one(params: &'a PaddedMontyParams) -> Self {
        Self {
            value: params.one.clone(),
            params,
        }
    }

    /// The value out of Montgomery form, below the modulus and at its
    /// width: a Montgomery multiplication by one takes the factor `R` away.
    /// Constant time.
    pub fn retrieve(&self) -> PaddedBigUint {
        let mut one = self.value.clone();
        one.set_one();
        product(self.value.as_limbs(), one.as_limbs(), self.params)
    }

    /// The parameters. Constant time.
    pub fn params(&self) -> &'a PaddedMontyParams {
        self.params
    }

    /// Panics when `rhs` has another modulus, or the same one at another
    /// width, in every build. Constant time apart from that panic: the
    /// widths are public, and the moduli are compared in constant time.
    pub(super) fn assert_same_modulus(&self, rhs: &Self) {
        let (lhs, rhs) = (&self.params.modulus, &rhs.params.modulus);
        let same = lhs.as_limbs().len() == rhs.as_limbs().len() && lhs == rhs;
        assert!(same, "operands of different moduli");
    }
}

/// `lhs · rhs · R⁻¹ mod m`, by one Montgomery multiplication into a new
/// buffer. Constant time.
pub(super) fn product(lhs: &[Limb], rhs: &[Limb], params: &PaddedMontyParams) -> PaddedBigUint {
    let mut output = vec![Limb::new(0); lhs.len()].into_boxed_slice();
    monty_mul(
        &mut output,
        lhs,
        rhs,
        params.modulus.as_limbs(),
        params.inverse,
    );
    PaddedBigUint::new(output)
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::{PaddedBigUint, Word};

    use super::super::testing::{MODULI, VALUES, form, int, params};
    use super::PaddedMontyForm;
    use crate::{Odd, PaddedMontyParams};

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
            assert_eq!(PaddedMontyForm::zero(&params).retrieve(), int(0), "{m}");
            assert_eq!(PaddedMontyForm::one(&params).retrieve(), int(1 % m), "{m}");
        }
    }

    #[test]
    fn every_value_takes_the_width_of_the_modulus() {
        // A narrow value takes the wider width of its modulus, and a value
        // wider than the modulus is cut down to its width once reduced.
        let (wide, narrow) = (
            (u128::BITS / Word::BITS) as usize,
            (u64::BITS / Word::BITS) as usize,
        );
        let wide_params = PaddedMontyParams::new(Odd::new(PaddedBigUint::from(255u128)).unwrap());
        let form = PaddedMontyForm::new(&PaddedBigUint::from(7u8), &wide_params);
        assert_eq!(
            (
                form.value.as_limbs().len(),
                form.retrieve().as_limbs().len()
            ),
            (wide, wide)
        );
        let narrow_params = params(255);
        let form = PaddedMontyForm::new(&PaddedBigUint::from(u128::MAX), &narrow_params);
        assert_eq!(
            (
                form.value.as_limbs().len(),
                form.retrieve().as_limbs().len()
            ),
            (narrow, narrow)
        );
        assert_eq!(
            form.retrieve(),
            PaddedBigUint::from((u128::MAX % 255) as u8)
        );
    }

    #[test]
    #[should_panic(expected = "operands of different moduli")]
    fn operands_of_different_moduli_panic() {
        let (three, five) = (params(3), params(5));
        let _ = form(1, &three) * form(1, &five);
    }

    #[test]
    #[should_panic(expected = "operands of different moduli")]
    fn the_same_modulus_at_another_width_panics() {
        let wide = PaddedMontyParams::new(Odd::new(PaddedBigUint::from(3u128)).unwrap());
        let narrow = params(3);
        let _ = form(1, &narrow) * PaddedMontyForm::new(&PaddedBigUint::from(1u8), &wide);
    }
}
