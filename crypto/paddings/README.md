# Block cipher paddings

The block-padding schemes ported from the Bouncy Castle C# source tree at
`crypto/src/crypto/paddings` are published as
[`tc_block_padding`](https://crates.io/crates/tc_block_padding): zero-byte,
PKCS#7, ANSI X9.23, ISO 10126-2, ISO 7816-4 and trailing bit complement.
`PaddedBufferedBlockCipher` is published in
[`tc_buffered_cipher`](https://crates.io/crates/tc_buffered_cipher).

The incubator crates that used to live here have been retired. The old padding
trait, `crypto/tc_pad`, remains only while `tc_iso9797_mac` depends on it.
