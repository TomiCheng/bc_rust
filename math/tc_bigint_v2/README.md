# tc_bigint_v2

Big integers for cryptography, `no_std`, in three kinds, each unsigned and
signed: fixed-width on the stack, padded to a width chosen at run time, and
arbitrary precision.

## Types

- `FixedBigUint<N>`, `FixedBigInt<N>`: `N` limbs on the stack, constant time, no allocator.
- `PaddedBigUint`, `PaddedBigInt`: a width chosen when built, on the heap, constant time.
- `BigUint`, `BigInt`: as many limbs as the value takes, variable time, for public values.
- `Limb`, `LimbArray<N>`, `Word`: the storage, a 64-bit word on 64-bit targets and 32-bit otherwise.

## Traits

- The operators, with an integer or a `u32` on the right, which panic on overflow in every build.
- From `num-traits`: `Zero`, `One`, `Num`, checked, wrapping, saturating and overflowing arithmetic, `Pow`, `Euclid`, `Signed`, `Unsigned`, and `Bounded` on the fixed-width types.
- `BitOps`, `Gcd`, and `ArrayEncoding` to and from words or bytes.
- `ConstantTimeEq`, `ConstantTimeOrd`, `ConditionallySelectable` and `Zeroize`.

## Features

- `alloc`: the padded and big types.

## Usage

```rust
use tc_bigint_v2::{FixedBigUint, Word};

type U256 = FixedBigUint<{ 256 / Word::BITS as usize }>;

let x = U256::from(u128::MAX);
let y = &x * &x + 7u32;
assert_eq!(y.div_rem(&x), (x, U256::from(7u8)));
```

## Security

- Constant time covers the fixed-width and padded types; the big ones are
  for public values only. Every method says in its doc which it is.
- The constant-time types wipe the working values they give up. The values
  they hand back are yours to wipe, through `Zeroize` or `Zeroizing`, and
  copies the compiler makes on its own are beyond reach.
- This is a learning port, with no independent security audit.
