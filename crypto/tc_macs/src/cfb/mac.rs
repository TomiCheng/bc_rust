use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{self, Display, Formatter};

use tc_block_cipher::{BlockCipher, BlockCipherInit};
use tc_block_modes::IvParams;
use tc_block_padding::BlockCipherPadding;

use super::shared::CfbMacCore;
use crate::{InitError, Mac, MacError, MacInit};

pub struct CfbMac<C> {
    core: CfbMacCore<C, Vec<u8>>,
}

impl<C: BlockCipher> CfbMac<C> {
    /// 預設 8-bit feedback、tag 半個 block，同 BC。
    pub fn new(cipher: C) -> Self {
        let mac_size = cipher.block_size() / 2;
        Self::with_sizes(cipher, 1, mac_size)
    }

    /// `feedback_size` 與 `mac_size` 以 byte 計；不在 `1..=block size` 時 panic，
    /// block size 為 0 也一樣。
    pub fn with_sizes(cipher: C, feedback_size: usize, mac_size: usize) -> Self {
        let block_size = cipher.block_size();
        let buffers = core::array::from_fn(|_| vec![0; block_size]);
        Self {
            core: CfbMacCore::new(cipher, buffers, feedback_size, mac_size),
        }
    }
}

impl<C: Display> Display for CfbMac<C> {
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

impl<C> Mac for CfbMac<C>
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

impl<C, P> MacInit<P> for CfbMac<C>
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

pub struct PaddedCfbMac<C, P> {
    mac: CfbMac<C>,
    padding: P,
}

impl<C: BlockCipher, P> PaddedCfbMac<C, P> {
    /// 預設 8-bit feedback、tag 半個 block，同 BC。
    pub fn new(cipher: C, padding: P) -> Self {
        Self {
            mac: CfbMac::new(cipher),
            padding,
        }
    }

    /// `feedback_size` 與 `mac_size` 以 byte 計；不在 `1..=block size` 時 panic，
    /// block size 為 0 也一樣。
    pub fn with_sizes(cipher: C, feedback_size: usize, mac_size: usize, padding: P) -> Self {
        Self {
            mac: CfbMac::with_sizes(cipher, feedback_size, mac_size),
            padding,
        }
    }
}

impl<C, P> PaddedCfbMac<C, P> {
    pub fn padding(&self) -> &P {
        &self.padding
    }
}

impl<C: Display, P> Display for PaddedCfbMac<C, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 的名稱不含 padding
        self.mac.fmt(f)
    }
}

impl<C, P> Mac for PaddedCfbMac<C, P>
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

impl<C, P, Q> MacInit<Q> for PaddedCfbMac<C, P>
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
