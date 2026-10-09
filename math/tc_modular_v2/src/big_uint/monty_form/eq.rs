//! Comparison and wiping of [`BigMontyForm`].

use tc_zeroize::Zeroize;

use super::BigMontyForm;

/// Equal when the moduli and the values both are. Variable time: only for
/// public values; secret ones go through [`PaddedMontyForm`], which compares
/// in constant time.
///
/// [`PaddedMontyForm`]: crate::PaddedMontyForm
impl PartialEq for BigMontyForm<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.params.modulus == other.params.modulus && self.value == other.value
    }
}

impl Eq for BigMontyForm<'_> {}

/// Overwrites the value, which leaves zero; the parameters are borrowed,
/// and left to their owner. Constant time.
impl Zeroize for BigMontyForm<'_> {
    fn zeroize(&mut self) {
        self.value.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use tc_bigint_v2::Limb;
    use tc_zeroize::Zeroize;

    use super::super::testing::{form, params};

    #[test]
    fn forms_are_equal_only_with_the_same_modulus_and_value() {
        let (eleven, thirteen) = (params(11), params(13));
        assert_eq!(form(7, &eleven), form(18, &eleven));
        assert_ne!(form(7, &eleven), form(8, &eleven));
        assert_ne!(form(7, &eleven), form(7, &thirteen));
    }

    #[test]
    fn zeroizing_clears_the_value() {
        let eleven = params(11);
        let mut zeroized = form(7, &eleven);
        zeroized.zeroize();
        assert!(zeroized.value.iter().all(|limb| *limb == Limb::new(0)));
    }
}
