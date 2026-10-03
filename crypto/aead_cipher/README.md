# AEAD ciphers

Authenticated encryption with associated data (AEAD), measured against the
AEAD engines and block-cipher modes in the Bouncy Castle C# source tree at
`crypto/src/crypto/engines` and `crypto/src/crypto/modes`.

## Graduated

These left the incubator and are published from
[TomiCheng/tc_aead_cipher](https://github.com/TomiCheng/tc_aead_cipher), on
contracts of their own rather than `tc_cipher`'s:

| Crate | Contents |
|-------|----------|
| `tc_aead_cipher` | The `AeadCipher` and `AeadCipherInit` contracts, and GCM, EAX, CCM, GCM-SIV, OCB3 and KCCM over any `tc_block_cipher` engine |
| `tc_ascon_aead` | Ascon-AEAD128 (NIST SP 800-232) and the Ascon v1.2 variants |
| `tc_grain128_aead` | Grain-128AEAD with a fixed or `Vec`-backed associated-data buffer |
| `tc_sparkle_aead` | The four SCHWAEMM parameter sets, with an SSE2 permutation on x86 |

## Still here

The crates below still use the `tc_cipher::AeadCipher` contracts and stay
until nothing in the workspace needs them:

| Crate | Contents | Why it stays |
|-------|----------|--------------|
| `tc_chacha_aead` | ChaCha20-Poly1305 and XChaCha20-Poly1305 | Not ported yet; waits for the MAC contract |

## Verification

Run the remaining AEAD tests from the workspace root:

```bash
cargo test -p tc_chacha_aead --locked
```
