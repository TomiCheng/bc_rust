# tc_modular_v2

Modular arithmetic for cryptography over the integers of `tc_bigint_v2`,
`no_std`: the `Mod*` operations, and Montgomery parameters and forms for an
odd modulus.

## Types

- `NonZero<T>`, `Odd<T>`: a value checked once to be non-zero, or odd, as a modulus must be.
- `FixedMontyParams<N>`, `PaddedMontyParams`, `BigMontyParams`: what Montgomery arithmetic modulo an odd modulus works out once.
- `FixedMontyForm<N>`, `PaddedMontyForm`, `BigMontyForm`: a value kept as `x · R mod m`, with the operators, `square`, `double`, `pow` and `invert`.

## Traits

- `ModAdd`, `ModSub`, `ModMul`, `ModPow` and `ModInverse`, on `FixedBigUint<N>`, `PaddedBigUint` and `BigUint`, over a `NonZero` modulus.

## Features

- `alloc`: the padded and big types.

## Usage

```rust
use tc_bigint_v2::{FixedBigUint, Word};
use tc_modular_v2::{FixedMontyForm, FixedMontyParams, ModInverse, NonZero, Odd};

type U256 = FixedBigUint<{ 256 / Word::BITS as usize }>;

let p = U256::from(i128::MAX as u128); // 2^127 - 1, a prime
let params = FixedMontyParams::new(Odd::new(p.clone()).unwrap());
let three = FixedMontyForm::new(&U256::from(3u8), params);
assert_eq!(three.pow(&(&p - 1u32)).retrieve(), U256::from(1u8));
let inverse = U256::from(3u8).mod_inverse(&NonZero::new(p).unwrap());
assert_eq!(inverse, Some(U256::from(u128::MAX / 3)));
```

## Security

- Over the fixed-width and padded integers everything is constant time,
  but for two things that show: the parity of a modulus, which picks the
  path of `ModPow` and `ModInverse`, and whether an inverse exists, in the
  `Option` that holds it.
- Over `BigUint`, the way in and out of a form is variable time: for
  public values only.
- Working values are wiped; the values handed back are yours to wipe,
  through `Zeroize` or `Zeroizing`.
- This is a learning port, with no independent security audit.
