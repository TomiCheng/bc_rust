use core::fmt::{self, Display, Formatter};

use tc_block_cipher::{BlockCipher, BlockCipherInit};
use tc_block_modes::{FixedCbcBlockCipher, IvParams};
use tc_block_padding::BlockCipherPadding;

use super::shared::CbcMacCore;
use crate::{Mac, MacError, MacInit, MacInitError};

pub struct FixedCbcMac<C, const N: usize> {
    core: CbcMacCore<FixedCbcBlockCipher<C, N>, [u8; N]>,
}

impl<C, const N: usize> FixedCbcMac<C, N> {
    /// tag 預設半個 block，同 BC。
    pub fn new(cipher: C) -> Self {
        Self::with_mac_size(cipher, N / 2)
    }

    /// `mac_size` 以 byte 計；不在 `1..=N` 時 panic。
    pub fn with_mac_size(cipher: C, mac_size: usize) -> Self {
        Self {
            core: CbcMacCore::new(FixedCbcBlockCipher::new(cipher), [0; N], [0; N], mac_size),
        }
    }
}

impl<C: Display, const N: usize> Display for FixedCbcMac<C, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 的 AlgorithmName 就是 CBC 模式的名稱，例如 "AES/CBC"
        self.core.mode().fmt(f)
    }
}

impl<C, const N: usize> Mac for FixedCbcMac<C, N>
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

impl<C, P, const N: usize> MacInit<P> for FixedCbcMac<C, N>
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

pub struct FixedPaddedCbcMac<C, const N: usize, P> {
    mac: FixedCbcMac<C, N>,
    padding: P,
}

impl<C, const N: usize, P> FixedPaddedCbcMac<C, N, P> {
    /// tag 預設半個 block，同 BC。
    pub fn new(cipher: C, padding: P) -> Self {
        Self::with_mac_size(cipher, N / 2, padding)
    }

    /// `mac_size` 以 byte 計；不在 `1..=N` 時 panic。
    pub fn with_mac_size(cipher: C, mac_size: usize, padding: P) -> Self {
        Self {
            mac: FixedCbcMac::with_mac_size(cipher, mac_size),
            padding,
        }
    }

    pub fn padding(&self) -> &P {
        &self.padding
    }
}

impl<C: Display, const N: usize, P> Display for FixedPaddedCbcMac<C, N, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 的名稱不含 padding
        self.mac.fmt(f)
    }
}

impl<C, const N: usize, P> Mac for FixedPaddedCbcMac<C, N, P>
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

impl<C, const N: usize, P, Q> MacInit<Q> for FixedPaddedCbcMac<C, N, P>
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
