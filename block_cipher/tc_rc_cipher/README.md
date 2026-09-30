# tc_rc_cipher

RC2, RC5 and RC6 single-block engines on the `tc_block_cipher` interfaces, in
an allocator-free `no_std` crate. Everything is exported at the crate root:
the engines, the parameter traits `Rc2Params` and `Rc5Params`, their borrowed
implementations `Rc2ParamsRef` and `Rc5ParamsRef`, and constants named after
their algorithm, such as `RC2_BLOCK_BYTES`. RC4 is a stream
cipher and is not part of this crate.

These ciphers are provided for interoperability with existing formats; no
mode, padding or authentication is provided.

## RC2

`Rc2TableEngine` takes 1- to 128-byte keys, an 8-byte block and an
independently selected effective key size from 1 to 1024 bits.

**Variable time:** secret bytes index the PI table during key setup, and mash
rounds index subkeys with block data. Use only where cache-timing leakage is
outside the threat model; there is no constant-time alternative. Stored
schedules are wiped on drop, but caller buffers and all temporary copies are not.

## RC5

`Rc532Engine` and `Rc564Engine` take 1- to 255-byte keys and 0 to
255 rounds. Their blocks are 8 and 16 bytes respectively.

**Constant time** on mainstream x86, x86-64 and AArch64 processors with
operand-independent rotations; processors without a barrel shifter can leak.
Stored schedules are wiped on replacement and drop, but caller buffers and
all register or stack copies are not guaranteed to be erased.

## RC6

`Rc6Engine` is RC6-32/20 with 1- to 255-byte keys and 16-byte blocks.

**Constant time under hardware assumptions:** data-dependent rotations and
32-bit multiplications must have fixed latency, as on mainstream x86, x86-64
and AArch64. Processors without a barrel shifter or with early-terminating
multipliers can leak secrets. Stored schedules are wiped on drop; caller
buffers and every register or stack copy are not.

## Usage

```rust
use tc_block_cipher::{BlockCipher, BlockCipherInit, CipherDirection};
use tc_rc_cipher::{Rc532Engine, Rc5ParamsRef};

let mut engine = Rc532Engine::new();
engine.init(CipherDirection::Encrypt, &Rc5ParamsRef::with_default_rounds(&[0x42; 16]))?;
let mut output = [0; 8];
engine.process_block(&[0x11; 8], &mut output)?;
# Ok::<(), Box<dyn core::error::Error>>(())
```
