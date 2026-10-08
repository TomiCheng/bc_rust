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

use tc_bigint_v2::{BigInt, BigUint, FixedBigUint, PaddedBigInt, PaddedBigUint};

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

    // negation works in its operand; only the borrowed form clones
    let padded_int = || PaddedBigInt::from(-5i128);
    assert_eq!(allocations_of(padded_int, |a| -a), 0);
    assert_eq!(allocations_of(padded_int, |a| -&a), 1);
    let int = || BigInt::from(-5i64);
    assert_eq!(allocations_of(int, |a| -a), 0);
    assert_eq!(allocations_of(int, |a| -&a), 1);

    // the Fixed types never touch the heap
    let fixed = || (FixedBigUint::<4>::from(5u8), FixedBigUint::<4>::from(6u8));
    assert_eq!(allocations_of(fixed, |(a, b)| &a + &b), 0);
    assert_eq!(allocations_of(fixed, |(a, b)| a ^ b), 0);
}
