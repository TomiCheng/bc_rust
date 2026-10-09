//! Constant-time comparison, selection and wiping of [`FixedMontyForm`].

use tc_bigint_v2::FixedBigUint;
use tc_constant_time::{Choice, ConditionallySelectable, ConstantTimeEq};
use tc_zeroize::Zeroize;

use super::FixedMontyForm;

/// Equal when the moduli and the values both are. Constant time.
impl<const N: usize> ConstantTimeEq for FixedMontyForm<N> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.params.modulus.ct_eq(&other.params.modulus) & self.value.ct_eq(&other.value)
    }
}

/// Through `ct_eq`. Constant time.
impl<const N: usize> PartialEq for FixedMontyForm<N> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl<const N: usize> Eq for FixedMontyForm<N> {}

/// The value of `a` or of `b`, as the choice says; the parameters are the
/// same either way. Panics when the moduli differ, in every build. Constant
/// time, apart from that panic.
impl<const N: usize> ConditionallySelectable for FixedMontyForm<N> {
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
        FixedBigUint::conditional_swap(&mut a.value, &mut b.value, choice);
    }
}

/// Overwrites the value and the copy of the parameters, which leaves the
/// form of no use. Constant time.
impl<const N: usize> Zeroize for FixedMontyForm<N> {
    fn zeroize(&mut self) {
        self.value.zeroize();
        self.params.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use tc_constant_time::{Choice, ConditionallySelectable};
    use tc_zeroize::Zeroize;

    use super::super::testing::{form, int};
    use super::FixedMontyForm;

    #[test]
    fn forms_are_equal_only_with_the_same_modulus_and_value() {
        assert_eq!(form(7, 11), form(18, 11));
        assert_ne!(form(7, 11), form(8, 11));
        assert_ne!(form(7, 11), form(7, 13));
    }

    #[test]
    fn selecting_follows_the_choice() {
        let (a, b) = (form(2, 11), form(5, 11));
        for bit in [0, 1] {
            let chosen = if bit == 1 { &b } else { &a };
            let selected = FixedMontyForm::conditional_select(&a, &b, Choice::from_lsb(bit));
            assert_eq!(&selected, chosen);
        }
    }

    #[test]
    fn zeroizing_clears_the_value_and_the_parameters() {
        let mut zeroized = form(7, 11);
        zeroized.zeroize();
        assert_eq!(zeroized.value, int(0));
        assert_eq!(zeroized.params.modulus, int(0));
    }
}
