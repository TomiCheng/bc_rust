//! Counts the allocations of the binary operators, to hold them to the
//! storage they are given.
//!
//! An operand passed by value lends its storage to the result, so only
//! `&a op &b` may allocate, and the heap types grow only when the result is
//! wider than the operand whose storage they reuse. The counter is global,
//! so this file is its own test binary with a single test, which keeps other
//! tests from counting into it.

#![cfg(feature = "alloc")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use num_traits::{One, Zero};
use tc_bigint_v2::{BigInt, BigUint, FixedBigUint, PaddedBigInt, PaddedBigUint, Word};

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

/// Forwards to the system allocator and counts each allocation; a
/// reallocation goes through the default `realloc`, so it counts too.
struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// The allocations made while `action` runs; its operands are built
/// beforehand and its result is dropped afterwards.
fn allocations_of<T, R>(operands: impl FnOnce() -> T, action: impl FnOnce(T) -> R) -> usize {
    let operands = operands();
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    let result = action(operands);
    let count = ALLOCATIONS.load(Ordering::Relaxed) - before;
    drop(result);
    count
}

#[test]
fn operators_reuse_the_storage_of_operands_passed_by_value() {
    // Padded at one width: only &a op &b allocates
    let padded = || (PaddedBigUint::from(5u128), PaddedBigUint::from(6u128));
    assert_eq!(allocations_of(padded, |(a, b)| a + b), 0);
    assert_eq!(allocations_of(padded, |(a, b)| a + &b), 0);
    assert_eq!(allocations_of(padded, |(a, b)| &a + b), 0);
    assert_eq!(allocations_of(padded, |(a, b)| &a + &b), 1);
    assert_eq!(
        allocations_of(padded, |(mut a, b)| {
            a += &b;
            a
        }),
        0
    );
    assert_eq!(
        allocations_of(padded, |(mut a, b)| {
            a += b;
            a
        }),
        0
    );
    assert_eq!(allocations_of(padded, |(a, b)| a ^ b), 0);
    assert_eq!(allocations_of(padded, |(a, b)| &a & &b), 1);

    // Padded across widths, a the narrower: working in a grows it once
    let mixed = || (PaddedBigUint::from(5u8), PaddedBigUint::from(6u128));
    assert_eq!(allocations_of(mixed, |(a, b)| a + &b), 1);
    assert_eq!(allocations_of(mixed, |(a, b)| &b + a), 1);
    // working in b, the wider one, grows nothing
    assert_eq!(allocations_of(mixed, |(a, b)| &a + b), 0);
    // &a op &b copies at the wider width in one go, whichever side is wider
    assert_eq!(allocations_of(mixed, |(a, b)| &a + &b), 1);
    assert_eq!(allocations_of(mixed, |(a, b)| &b + &a), 1);
    assert_eq!(allocations_of(mixed, |(a, b)| &a | &b), 1);
    let signed = || (PaddedBigInt::from(-5i128), PaddedBigInt::from(6i128));
    assert_eq!(allocations_of(signed, |(a, b)| a + b), 0);

    // BigUint and BigInt grow only when the sum needs another limb
    let small = || (BigUint::from(5u8), BigUint::from(6u8));
    assert_eq!(allocations_of(small, |(a, b)| a + b), 0);
    assert_eq!(allocations_of(small, |(a, b)| &a + b), 0);
    assert_eq!(allocations_of(small, |(a, b)| &a + &b), 1);
    let carrying = || (BigUint::from(u128::MAX), BigUint::from(1u8));
    assert_eq!(allocations_of(carrying, |(a, b)| a + b), 1);
    // the copy for &a + &b already has room for the carry
    assert_eq!(allocations_of(carrying, |(a, b)| &a + &b), 1);
    let negative = || (BigInt::from(i128::MIN), BigInt::from(i128::MIN));
    assert_eq!(allocations_of(negative, |(a, b)| &a + &b), 1);
    let ints = || (BigInt::from(-5i64), BigInt::from(6i64));
    assert_eq!(allocations_of(ints, |(a, b)| a + b), 0);
    assert_eq!(allocations_of(ints, |(a, b)| a | b), 0);

    // subtraction works in its left operand, or in a copy of it
    let padded = || (PaddedBigUint::from(6u128), PaddedBigUint::from(5u128));
    assert_eq!(allocations_of(padded, |(a, b)| a - b), 0);
    assert_eq!(allocations_of(padded, |(a, b)| a - &b), 0);
    assert_eq!(allocations_of(padded, |(a, b)| &a - b), 1);
    assert_eq!(allocations_of(padded, |(a, b)| &a - &b), 1);
    // a the narrower: working in a grows it once, and its copy is made at
    // the wider width in one go
    let mixed = || (PaddedBigUint::from(6u8), PaddedBigUint::from(5u128));
    assert_eq!(allocations_of(mixed, |(a, b)| a - &b), 1);
    assert_eq!(allocations_of(mixed, |(a, b)| &a - &b), 1);
    let signed = || (PaddedBigInt::from(-5i128), PaddedBigInt::from(6i128));
    assert_eq!(allocations_of(signed, |(a, b)| a - b), 0);
    let small = || (BigUint::from(6u8), BigUint::from(5u8));
    assert_eq!(allocations_of(small, |(a, b)| a - b), 0);
    assert_eq!(allocations_of(small, |(a, b)| a - &b), 0);
    assert_eq!(allocations_of(small, |(a, b)| &a - b), 1);
    assert_eq!(allocations_of(small, |(a, b)| &a - &b), 1);
    let ints = || (BigInt::from(-5i64), BigInt::from(6i64));
    assert_eq!(allocations_of(ints, |(a, b)| a - b), 0);
    assert_eq!(allocations_of(ints, |(a, b)| &a - b), 1);
    // BigInt grows only when the difference needs another limb, and the
    // copy for &a - &b already has room for it
    let growing = || (BigInt::from(i128::MIN), BigInt::from(i128::MAX));
    assert_eq!(allocations_of(growing, |(a, b)| a - b), 1);
    assert_eq!(allocations_of(growing, |(a, b)| &a - &b), 1);
    assert_eq!(allocations_of(growing, |(a, b)| &a - b), 1);

    // multiplication works in place for Fixed and Padded, in its right
    // operand for &a * b; BigUint and BigInt build the product in a new
    // buffer of both lengths, so every form allocates once
    let padded = || (PaddedBigUint::from(5u128), PaddedBigUint::from(6u128));
    assert_eq!(allocations_of(padded, |(a, b)| a * b), 0);
    assert_eq!(allocations_of(padded, |(a, b)| a * &b), 0);
    assert_eq!(allocations_of(padded, |(a, b)| &a * b), 0);
    assert_eq!(allocations_of(padded, |(a, b)| &a * &b), 1);
    let signed = || (PaddedBigInt::from(-5i128), PaddedBigInt::from(6i128));
    assert_eq!(allocations_of(signed, |(a, b)| a * &b), 0);
    let small = || (BigUint::from(5u8), BigUint::from(6u8));
    assert_eq!(allocations_of(small, |(a, b)| a * b), 1);
    assert_eq!(allocations_of(small, |(a, b)| &a * &b), 1);
    let zero = || (BigUint::default(), BigUint::from(6u8));
    assert_eq!(allocations_of(zero, |(a, b)| a * &b), 0);
    let ints = || (BigInt::from(5i64), BigInt::from(6i64));
    assert_eq!(allocations_of(ints, |(a, b)| &a * &b), 1);
    // a negative operand is negated into a copy first
    let negative = || (BigInt::from(-5i64), BigInt::from(6i64));
    assert_eq!(allocations_of(negative, |(a, b)| &a * &b), 2);

    // Padded divides in place, the remainder worked out in a new buffer,
    // which % then keeps
    let padded = || (PaddedBigUint::from(200u128), PaddedBigUint::from(7u128));
    assert_eq!(allocations_of(padded, |(a, b)| a / &b), 1);
    assert_eq!(allocations_of(padded, |(a, b)| a % &b), 1);
    assert_eq!(allocations_of(padded, |(a, b)| &a / &b), 2);
    assert_eq!(allocations_of(padded, |(a, b)| a.div_rem(&b)), 2);
    // BigUint divides in the storage of the dividend; a divisor of one limb
    // needs only the limb of the remainder besides
    let small = || (BigUint::from(u128::MAX), BigUint::from(7u8));
    assert_eq!(allocations_of(small, |(a, b)| a / &b), 1);
    assert_eq!(allocations_of(small, |(a, b)| a % &b), 1);

    // shifts work in place; a left shift grows only by the limbs it needs
    let value = || BigUint::from(5u8);
    assert_eq!(allocations_of(value, |a| a << 3), 0);
    assert_eq!(allocations_of(value, |a| a >> 1), 0);
    assert_eq!(allocations_of(value, |a| a << Word::BITS), 1);
    // the copy for &a << n already has room for the result
    assert_eq!(allocations_of(value, |a| &a << Word::BITS), 1);
    assert_eq!(allocations_of(value, |a| &a >> 1), 1);
    let int_value = || BigInt::from(-5i64);
    assert_eq!(allocations_of(int_value, |a| a << 3), 0);
    assert_eq!(allocations_of(int_value, |a| &a << Word::BITS), 1);
    let padded_value = || PaddedBigUint::from(5u128);
    assert_eq!(allocations_of(padded_value, |a| a << 3), 0);
    assert_eq!(allocations_of(padded_value, |a| &a >> 3), 1);

    // negation works in its operand; only the borrowed form clones
    let padded_int = || PaddedBigInt::from(-5i128);
    assert_eq!(allocations_of(padded_int, |a| -a), 0);
    assert_eq!(allocations_of(padded_int, |a| -&a), 1);
    let int = || BigInt::from(-5i64);
    assert_eq!(allocations_of(int, |a| -a), 0);
    assert_eq!(allocations_of(int, |a| -&a), 1);

    // zero needs no storage, and setting a value to zero keeps its storage
    // for later use
    assert_eq!(allocations_of(|| (), |()| PaddedBigUint::zero()), 0);
    assert_eq!(allocations_of(|| (), |()| BigUint::zero()), 0);
    let reset = || (PaddedBigUint::from(5u128), PaddedBigUint::from(6u128));
    assert_eq!(
        allocations_of(reset, |(mut a, b)| {
            a.set_zero();
            a += &b;
            a
        }),
        0
    );
    let reset = || (BigUint::from(5u8), BigUint::from(6u8));
    assert_eq!(
        allocations_of(reset, |(mut a, b)| {
            a.set_zero();
            a += &b;
            a
        }),
        0
    );

    // checking for one builds no one, and setting a value to one keeps its
    // storage
    let padded_value = || PaddedBigUint::from(5u128);
    assert_eq!(allocations_of(padded_value, |a| a.is_one()), 0);
    assert_eq!(
        allocations_of(padded_value, |mut a| {
            a.set_one();
            a
        }),
        0
    );
    let value = || BigUint::from(5u8);
    assert_eq!(allocations_of(value, |a| a.is_one()), 0);
    assert_eq!(
        allocations_of(value, |mut a| {
            a.set_one();
            a
        }),
        0
    );

    // the complement works in its operand; only the borrowed form copies
    let padded_value = || PaddedBigUint::from(5u128);
    assert_eq!(allocations_of(padded_value, |a| !a), 0);
    assert_eq!(allocations_of(padded_value, |a| !&a), 1);
    let int_value = || BigInt::from(-5i64);
    assert_eq!(allocations_of(int_value, |a| !a), 0);
    assert_eq!(allocations_of(int_value, |a| !&a), 1);

    // conversions between the heap types take the storage they are given
    let padded_value = || PaddedBigUint::from(5u128);
    assert_eq!(allocations_of(padded_value, BigUint::from), 0);
    let value = || BigUint::from(5u8);
    assert_eq!(allocations_of(value, PaddedBigUint::from), 0);
    let int_value = || BigInt::from(5i64);
    assert_eq!(allocations_of(int_value, BigUint::try_from), 0);

    // a word on the right needs no storage of its own: BigUint works in its
    // left operand, or in a copy of it, and grows only when the result needs
    // another limb
    let small = || BigUint::from(6u8);
    assert_eq!(allocations_of(small, |a| a + 1u32), 0);
    assert_eq!(allocations_of(small, |a| 1u32 + a), 0);
    assert_eq!(allocations_of(small, |a| a - 1u32), 0);
    assert_eq!(allocations_of(small, |a| a * 3u32), 0);
    assert_eq!(allocations_of(small, |a| a / 4u32), 0);
    assert_eq!(allocations_of(small, |a| a % 4u32), 0);
    assert_eq!(allocations_of(small, |a| &a + 1u32), 1);
    let carrying = || BigUint::from(u128::MAX);
    assert_eq!(allocations_of(carrying, |a| a + 1u32), 1);
    // Padded keeps its width and works in place; BigInt grows only when the
    // result needs another limb
    let padded = || PaddedBigUint::from(6u128);
    assert_eq!(allocations_of(padded, |a| a + 1u32), 0);
    assert_eq!(allocations_of(padded, |a| a * 3u32), 0);
    assert_eq!(allocations_of(padded, |a| a % 4u32), 0);
    assert_eq!(allocations_of(padded, |a| &a + 1u32), 1);
    let signed = || PaddedBigInt::from(-6i128);
    assert_eq!(allocations_of(signed, |a| a - 1u32), 0);
    assert_eq!(allocations_of(signed, |a| a / 4u32), 0);
    let int = || BigInt::from(-6i64);
    assert_eq!(allocations_of(int, |a| a + 1u32), 0);
    assert_eq!(allocations_of(int, |a| a * 3u32), 0);
    assert_eq!(allocations_of(int, |a| a / 4u32), 0);

    // the Fixed types never touch the heap
    let fixed = || (FixedBigUint::<4>::from(5u8), FixedBigUint::<4>::from(6u8));
    assert_eq!(allocations_of(fixed, |(a, b)| &a + &b), 0);
    assert_eq!(allocations_of(fixed, |(a, b)| a ^ b), 0);
    assert_eq!(allocations_of(fixed, |(a, b)| &b - &a), 0);
    assert_eq!(allocations_of(fixed, |(a, _)| &a << 3), 0);
    assert_eq!(allocations_of(fixed, |(a, b)| &a * &b), 0);
    assert_eq!(allocations_of(fixed, |(a, b)| &b / &a), 0);
    assert_eq!(allocations_of(fixed, |(a, b)| b.div_rem(&a)), 0);
}
