use core::fmt::{self, Display, Formatter};

use tc_block_cipher::{BlockCipher, BlockCipherInit};
use tc_block_modes::IvParams;
use tc_block_padding::BlockCipherPadding;

use super::shared::CfbMacCore;
use crate::{Mac, MacError, MacInit, MacInitError};

pub struct FixedCfbMac<C, const N: usize> {
    core: CfbMacCore<C, [u8; N]>,
}

impl<C, const N: usize> FixedCfbMac<C, N> {
    /// 預設 8-bit feedback、tag 半個 block，同 BC。
    pub fn new(cipher: C) -> Self {
        Self::with_sizes(cipher, 1, N / 2)
    }

    /// `feedback_size` 與 `mac_size` 以 byte 計；不在 `1..=N` 時 panic。
    pub fn with_sizes(cipher: C, feedback_size: usize, mac_size: usize) -> Self {
        Self {
            core: CfbMacCore::new(cipher, [[0; N]; 4], feedback_size, mac_size),
        }
    }
}

impl<C: Display, const N: usize> Display for FixedCfbMac<C, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 的名稱是 MacCFBBlockCipher 的名稱，例如 "DES/CFB8"
        write!(
            f,
            "{}/CFB{}",
            self.core.cipher(),
            self.core.segment_size() * 8
        )
    }
}

impl<C, const N: usize> Mac for FixedCfbMac<C, N>
where
    C: BlockCipher,
    C::Error: 'static,
{
    type Error = MacError<C::Error>;

    fn mac_size(&self) -> usize {
        self.core.mac_size()
    }

    fn update(&mut self, input: &[u8]) -> Result<(), Self::Error> {
        self.core.update(input)
    }

    fn do_final(&mut self, output: &mut [u8]) -> Result<usize, Self::Error> {
        self.core.do_final(output)
    }

    fn reset(&mut self) {
        self.core.clear_message();
    }
}

impl<C, P, const N: usize> MacInit<P> for FixedCfbMac<C, N>
where
    C: BlockCipher + BlockCipherInit<P>,
    P: IvParams + ?Sized,
    <C as BlockCipherInit<P>>::Error: 'static,
{
    type Error = MacInitError<<C as BlockCipherInit<P>>::Error>;

    fn init(&mut self, params: &P) -> Result<(), Self::Error> {
        self.core.init(params)
    }
}

pub struct FixedPaddedCfbMac<C, const N: usize, P> {
    mac: FixedCfbMac<C, N>,
    padding: P,
}

impl<C, const N: usize, P> FixedPaddedCfbMac<C, N, P> {
    /// 預設 8-bit feedback、tag 半個 block，同 BC。
    pub fn new(cipher: C, padding: P) -> Self {
        Self::with_sizes(cipher, 1, N / 2, padding)
    }

    /// `feedback_size` 與 `mac_size` 以 byte 計；不在 `1..=N` 時 panic。
    pub fn with_sizes(cipher: C, feedback_size: usize, mac_size: usize, padding: P) -> Self {
        Self {
            mac: FixedCfbMac::with_sizes(cipher, feedback_size, mac_size),
            padding,
        }
    }

    pub fn padding(&self) -> &P {
        &self.padding
    }
}

impl<C: Display, const N: usize, P> Display for FixedPaddedCfbMac<C, N, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 的名稱不含 padding
        self.mac.fmt(f)
    }
}

impl<C, const N: usize, P> Mac for FixedPaddedCfbMac<C, N, P>
where
    C: BlockCipher,
    C::Error: 'static,
    P: BlockCipherPadding,
{
    type Error = MacError<C::Error>;

    fn mac_size(&self) -> usize {
        self.mac.mac_size()
    }

    fn update(&mut self, input: &[u8]) -> Result<(), Self::Error> {
        self.mac.update(input)
    }

    fn do_final(&mut self, output: &mut [u8]) -> Result<usize, Self::Error> {
        self.mac.core.do_final_padded(&mut self.padding, output)
    }

    fn reset(&mut self) {
        self.mac.reset();
    }
}

impl<C, const N: usize, P, Q> MacInit<Q> for FixedPaddedCfbMac<C, N, P>
where
    C: BlockCipher + BlockCipherInit<Q>,
    Q: IvParams + ?Sized,
    <C as BlockCipherInit<Q>>::Error: 'static,
{
    type Error = MacInitError<<C as BlockCipherInit<Q>>::Error>;

    fn init(&mut self, params: &Q) -> Result<(), Self::Error> {
        self.mac.init(params)
    }
}
