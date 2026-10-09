//! Wiping of the working values an operation leaves behind.

use tc_zeroize::Zeroize;

/// Puts `value` in `target` and wipes the value it held before giving up
/// its storage. Constant time.
pub(crate) fn replace_wiped<T: Zeroize>(target: &mut T, value: T) {
    let mut previous = core::mem::replace(target, value);
    previous.zeroize();
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use tc_zeroize::Zeroize;

    use super::replace_wiped;

    /// Notes when it is wiped.
    struct Probe<'a>(&'a Cell<bool>);

    impl Zeroize for Probe<'_> {
        fn zeroize(&mut self) {
            self.0.set(true);
        }
    }

    #[test]
    fn the_replaced_value_is_wiped_and_the_new_one_kept() {
        let (old, new) = (Cell::new(false), Cell::new(false));
        let mut target = Probe(&old);
        replace_wiped(&mut target, Probe(&new));
        assert!(old.get() && !new.get());
        assert!(core::ptr::eq(target.0, &new));
    }
}
