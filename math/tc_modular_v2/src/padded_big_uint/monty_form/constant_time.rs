//! Constant-time comparison, selection and wiping of [`PaddedMontyForm`].

use tc_bigint_v2::PaddedBigUint;
use tc_constant_time::{Choice, ConditionallySelectable, ConstantTimeEq};
use tc_zeroize::Zeroize;

use super::PaddedMontyForm;

/// Equal when the moduli and the values both are. Constant time.
impl ConstantTimeEq for PaddedMontyForm<'_> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.params.modulus.ct_eq(&other.params.modulus) & self.value.ct_eq(&other.value)
    }
}

/// Through `ct_eq`. Constant time.
impl PartialEq for PaddedMontyForm<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl Eq for PaddedMontyForm<'_> {}

/// The value of `a` or of `b`, as the choice says; the parameters are the
/// same either way. Panics when the moduli differ, in every build. Constant
/// time, apart from that panic.
impl ConditionallySelectable for PaddedMontyForm<'_> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut selected = a.clone();
        selected.conditional_assign(b, choice);
        selected
    }

    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        self.assert_same_modulus(other);
        self.value.conditional_assign(&other.value, choice);
    }

    fn conditional_swap(a: &mut Self, b: &mut Self, choice: Choice) {
        a.assert_same_modulus(b);
        PaddedBigUint::conditional_swap(&mut a.value, &mut b.value, choice);
    }
}

/// Overwrites the value, which leaves zero; the parameters are borrowed,
/// and left to their owner. Constant time.
impl Zeroize for PaddedMontyForm<'_> {
    fn zeroize(&mut self) {
        self.value.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;
    use tc_constant_time::{Choice, ConditionallySelectable};
    use tc_zeroize::Zeroize;

    use super::super::testing::{form, params};
    use super::PaddedMontyForm;

    #[test]
    fn forms_are_equal_only_with_the_same_modulus_and_value() {
        let (eleven, thirteen) = (params(11), params(13));
        assert_eq!(form(7, &eleven), form(18, &eleven));
        assert_ne!(form(7, &eleven), form(8, &eleven));
        assert_ne!(form(7, &eleven), form(7, &thirteen));
    }

    #[test]
    fn selecting_follows_the_choice() {
        let eleven = params(11);
        let (a, b) = (form(2, &eleven), form(5, &eleven));
        for bit in [0, 1] {
            let chosen = if bit == 1 { &b } else { &a };
            let selected = PaddedMontyForm::conditional_select(&a, &b, Choice::from_lsb(bit));
            assert_eq!(&selected, chosen);
        }
    }

    #[test]
    fn zeroizing_clears_the_value() {
        let eleven = params(11);
        let mut zeroized = form(7, &eleven);
        zeroized.zeroize();
        assert!(zeroized.value.is_zero());
    }
}
