//! 不需 allocator 的 HMAC。

use core::convert::Infallible;
use core::fmt::{self, Display, Formatter};

use tc_block_cipher::KeyParams;
use tc_digest::Digest;
use tc_zeroize::Zeroizing;

use crate::{Mac, MacError, MacInit};

const IPAD: u8 = 0x36;
const OPAD: u8 = 0x5c;
const MINIMUM_BLOCK_LENGTH: usize = 16;
/// 金鑰比 block 長時要先 hash，結果暫放在這麼大的區域陣列。
const MAXIMUM_DIGEST_SIZE: usize = 128;

/// HMAC over a cloneable digest `D`, without an allocator.
///
/// Instead of the pads it keeps the digest states after absorbing each pad,
/// as Bouncy Castle does for memoable digests, so a message restarts by
/// cloning a state rather than hashing a pad block again. The inner hash is
/// held in the caller's output buffer until the outer hash overwrites it.
///
/// The stored states are as sensitive as the key, and they are wiped only if
/// the digest wipes itself on drop.
pub struct FixedHmac<D> {
    digest: D,
    // 吸收完 ipad 與 opad 之後的狀態
    inner: D,
    outer: D,
    digest_size: usize,
    block_length: usize,
    initialized: bool,
}

impl<D: Digest + Clone> FixedHmac<D> {
    /// block 長度用 digest 回報的值。
    ///
    /// # Panics
    ///
    /// block 長度小於 16 bytes、小於 digest 輸出，或 digest 輸出超過 128 bytes 時。
    pub fn new(digest: D) -> Self {
        let block_length = digest.byte_length();
        Self::with_block_length(digest, block_length)
    }

    /// 對應 BC 可指定 block 長度的建構子。
    ///
    /// # Panics
    ///
    /// 同 [`new`](Self::new)。
    pub fn with_block_length(digest: D, block_length: usize) -> Self {
        let digest_size = digest.digest_size();
        assert!(
            block_length >= MINIMUM_BLOCK_LENGTH,
            "HMAC block length must be at least 16 bytes"
        );
        assert!(
            digest_size <= block_length,
            "HMAC digest size must not exceed its block length"
        );
        assert!(
            digest_size <= MAXIMUM_DIGEST_SIZE,
            "FixedHmac supports digests of up to 128 bytes"
        );
        Self {
            inner: digest.clone(),
            outer: digest.clone(),
            digest,
            digest_size,
            block_length,
            initialized: false,
        }
    }
}

impl<D> FixedHmac<D> {
    pub const fn underlying_digest(&self) -> &D {
        &self.digest
    }
}

impl<D: Display> Display for FixedHmac<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 的名稱是 digest 名稱加 "/HMAC"
        write!(f, "{}/HMAC", self.digest)
    }
}

impl<D: Digest + Clone> Mac for FixedHmac<D> {
    type Error = MacError;

    fn mac_size(&self) -> usize {
        self.digest_size
    }

    fn update(&mut self, input: &[u8]) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MacError::NotInitialised);
        }
        self.digest.update(input);
        Ok(())
    }

    fn do_final(&mut self, output: &mut [u8]) -> Result<usize, Self::Error> {
        if !self.initialized {
            return Err(MacError::NotInitialised);
        }
        let available = output.len();
        let output = output
            .get_mut(..self.digest_size)
            .ok_or(MacError::OutputTooShort {
                required: self.digest_size,
                available,
            })?;

        // 內層 hash 先暫放在 output，外層 hash 再覆寫成 tag
        let inner_length = self.digest.do_final(output);
        debug_assert_eq!(inner_length, self.digest_size);
        self.digest.clone_from(&self.outer);
        self.digest.update(output);
        let written = self.digest.do_final(output);
        debug_assert_eq!(written, self.digest_size);

        self.digest.clone_from(&self.inner);
        Ok(written)
    }

    fn reset(&mut self) {
        if self.initialized {
            self.digest.clone_from(&self.inner);
        } else {
            self.digest.reset();
        }
    }
}

impl<D, P> MacInit<P> for FixedHmac<D>
where
    D: Digest + Clone,
    P: KeyParams + ?Sized,
{
    type Error = Infallible;

    fn init(&mut self, params: &P) -> Result<(), Self::Error> {
        self.initialized = false;
        let mut hashed = Zeroizing::new([0u8; MAXIMUM_DIGEST_SIZE]);
        let mut key = params.key();
        if key.len() > self.block_length {
            self.digest.reset();
            self.digest.update(key);
            let written = self.digest.do_final(&mut hashed[..]);
            key = &hashed[..written];
        }

        // pad 不另存，逐 byte 餵進兩個狀態；金鑰長度是公開值
        self.inner.reset();
        self.outer.reset();
        for index in 0..self.block_length {
            let byte = key.get(index).copied().unwrap_or(0);
            self.inner.update_byte(byte ^ IPAD);
            self.outer.update_byte(byte ^ OPAD);
        }

        self.digest.clone_from(&self.inner);
        self.initialized = true;
        Ok(())
    }
}
