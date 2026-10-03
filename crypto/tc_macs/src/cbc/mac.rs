use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{self, Display, Formatter};

use tc_block_cipher::{BlockCipher, BlockCipherInit};
use tc_block_modes::{CbcBlockCipher, IvParams};
use tc_block_padding::BlockCipherPadding;

use super::shared::CbcMacCore;
use crate::{InitError, Mac, MacError, MacInit};

pub struct CbcMac<C> {
    core: CbcMacCore<CbcBlockCipher<C>, Vec<u8>>,
}

impl<C: BlockCipher> CbcMac<C> {
    /// tag 預設半個 block，同 BC。
    pub fn new(cipher: C) -> Self {
        let mac_size = cipher.block_size() / 2;
        Self::with_mac_size(cipher, mac_size)
    }

    /// `mac_size` 以 byte 計；不在 `1..=block size` 時 panic，block size 為 0 也一樣。
    pub fn with_mac_size(cipher: C, mac_size: usize) -> Self {
        let block_size = cipher.block_size();
        Self {
            core: CbcMacCore::new(
                CbcBlockCipher::new(cipher),
                vec![0; block_size],
                vec![0; block_size],
                mac_size,
            ),
        }
    }
}

impl<C: Display> Display for CbcMac<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 的 AlgorithmName 就是 CBC 模式的名稱，例如 "AES/CBC"
        self.core.mode().fmt(f)
    }
}

impl<C> Mac for CbcMac<C>
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

impl<C, P> MacInit<P> for CbcMac<C>
where
    C: BlockCipher + BlockCipherInit<P>,
    P: IvParams + ?Sized,
    <C as BlockCipherInit<P>>::Error: 'static,
{
    type Error = InitError<<C as BlockCipherInit<P>>::Error>;

    fn init(&mut self, params: &P) -> Result<(), Self::Error> {
        self.core.init(params)
    }
}

pub struct PaddedCbcMac<C, P> {
    mac: CbcMac<C>,
    padding: P,
}

impl<C: BlockCipher, P> PaddedCbcMac<C, P> {
    /// tag 預設半個 block，同 BC。
    pub fn new(cipher: C, padding: P) -> Self {
        Self {
            mac: CbcMac::new(cipher),
            padding,
        }
    }

    /// `mac_size` 以 byte 計；不在 `1..=block size` 時 panic，block size 為 0 也一樣。
    pub fn with_mac_size(cipher: C, mac_size: usize, padding: P) -> Self {
        Self {
            mac: CbcMac::with_mac_size(cipher, mac_size),
            padding,
        }
    }
}

impl<C, P> PaddedCbcMac<C, P> {
    pub fn padding(&self) -> &P {
        &self.padding
    }
}

impl<C: Display, P> Display for PaddedCbcMac<C, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 的名稱不含 padding
        self.mac.fmt(f)
    }
}

impl<C, P> Mac for PaddedCbcMac<C, P>
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

impl<C, P, Q> MacInit<Q> for PaddedCbcMac<C, P>
where
    C: BlockCipher + BlockCipherInit<Q>,
    Q: IvParams + ?Sized,
    <C as BlockCipherInit<Q>>::Error: 'static,
{
    type Error = InitError<<C as BlockCipherInit<Q>>::Error>;

    fn init(&mut self, params: &Q) -> Result<(), Self::Error> {
        self.mac.init(params)
    }
}
