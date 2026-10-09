//! The modular inverse by the binary extended greatest common divisor, in
//! constant time.

use tc_bigint_v2::{Limb, Word};
use tc_zeroize::Zeroize;

use crate::limb::{
    add_masked, borrow_out, halve_masked, is_one, reduce_once, select_assign, sub_masked,
};

/// `value⁻¹ mod modulus`, for `value` below `modulus`, both of the width of
/// `zero`, which gives the storage of every working value; `None` when
/// there is no inverse.
///
/// Stein's binary algorithm, extended: `u` and `v` start at `value` and
/// `modulus`, and the coefficients keep `A·value - B·modulus = u` and
/// `D·modulus - C·value = v`. Each step takes the smaller of `u` and `v`
/// from the larger when both are odd, then halves the one that is even,
/// so that after twice the bits of the width `u` is their greatest common
/// divisor and `A` the inverse when that is one. It needs `value` or
/// `modulus` odd; when both are even, there is no inverse.
///
/// Every step does all of its work and keeps what it needs through masks.
/// Constant time, apart from whether there is an inverse, which the `Option`
/// shows.
pub(crate) fn binary_inverse<S>(value: &[Limb], modulus: &[Limb], zero: &S) -> Option<S>
where
    S: AsRef<[Limb]> + AsMut<[Limb]> + Clone,
{
    let start = |limbs: &[Limb]| {
        let mut started = zero.clone();
        started.as_mut().copy_from_slice(limbs);
        started
    };
    let one = |mut limbs: S| {
        limbs.as_mut()[0] = Limb::new(1);
        limbs
    };
    let (mut u, mut v) = (start(value), start(modulus));
    let (mut a, mut b) = (one(zero.clone()), zero.clone());
    let (mut c, mut d) = (zero.clone(), one(zero.clone()));
    let (mut first, mut second) = (zero.clone(), zero.clone());

    for _ in 0..2 * modulus.len() as u32 * Word::BITS {
        // When both are odd, the smaller comes off the larger, and the
        // coefficients follow.
        let both_odd = (low_bit(u.as_ref()) & low_bit(v.as_ref())).wrapping_neg();
        let v_below_u = borrow_out(v.as_ref(), u.as_ref()).wrapping_neg();
        first.as_mut().copy_from_slice(v.as_ref());
        sub_masked(first.as_mut(), u.as_ref(), Word::MAX);
        select_assign(v.as_mut(), first.as_ref(), both_odd & !v_below_u);
        first.as_mut().copy_from_slice(u.as_ref());
        sub_masked(first.as_mut(), v.as_ref(), Word::MAX);
        select_assign(u.as_mut(), first.as_ref(), both_odd & v_below_u);

        // A + C mod modulus and B + D mod value, reduced together, as the
        // invariants tie them.
        first.as_mut().copy_from_slice(a.as_ref());
        let carry = add_masked(first.as_mut(), c.as_ref(), Word::MAX);
        second.as_mut().copy_from_slice(first.as_ref());
        let below = sub_masked(second.as_mut(), modulus, Word::MAX);
        let reduce = (carry | (below ^ 1)).wrapping_neg();
        select_assign(first.as_mut(), second.as_ref(), reduce);
        select_assign(a.as_mut(), first.as_ref(), both_odd & v_below_u);
        select_assign(c.as_mut(), first.as_ref(), both_odd & !v_below_u);
        first.as_mut().copy_from_slice(b.as_ref());
        add_masked(first.as_mut(), d.as_ref(), Word::MAX);
        second.as_mut().copy_from_slice(first.as_ref());
        sub_masked(second.as_mut(), value, Word::MAX);
        select_assign(first.as_mut(), second.as_ref(), reduce);
        select_assign(b.as_mut(), first.as_ref(), both_odd & v_below_u);
        select_assign(d.as_mut(), first.as_ref(), both_odd & !v_below_u);

        // One of the two is even now; it is halved, and its coefficients
        // with it, made even first by adding the modulus and the value.
        halve_with(&mut u, &mut a, &mut b, value, modulus);
        halve_with(&mut v, &mut c, &mut d, value, modulus);
    }

    // A modulus of one takes every value to zero, which is its own inverse
    // there; the coefficient is reduced for it.
    reduce_once(a.as_mut(), 0, modulus);
    let either_odd = low_bit(value) | low_bit(modulus);
    let invertible = either_odd & (is_one(u.as_ref()) | is_one(modulus));
    // Every working value but the inverse is wiped, and the inverse too when
    // there is none.
    for working in [
        &mut u,
        &mut v,
        &mut b,
        &mut c,
        &mut d,
        &mut first,
        &mut second,
    ] {
        working.as_mut().zeroize();
    }
    if invertible == 1 {
        Some(a)
    } else {
        a.as_mut().zeroize();
        None
    }
}

/// Halves `x` when it is even, and with it the coefficients `p` and `q` of
/// its invariant, which are made even first, when either is odd, by adding
/// the modulus to `p` and the value to `q`. Constant time.
fn halve_with<S>(x: &mut S, p: &mut S, q: &mut S, value: &[Limb], modulus: &[Limb])
where
    S: AsRef<[Limb]> + AsMut<[Limb]>,
{
    let even = (low_bit(x.as_ref()) ^ 1).wrapping_neg();
    halve_masked(x.as_mut(), 0, even);
    let odd = (low_bit(p.as_ref()) | low_bit(q.as_ref())).wrapping_neg();
    let p_carry = add_masked(p.as_mut(), modulus, odd & even);
    let q_carry = add_masked(q.as_mut(), value, odd & even);
    halve_masked(p.as_mut(), p_carry, even);
    halve_masked(q.as_mut(), q_carry, even);
}

/// The lowest bit of `value`, zero or one. Constant time.
fn low_bit(value: &[Limb]) -> Word {
    value.first().map_or(0, |limb| limb.to_word() & 1)
}
