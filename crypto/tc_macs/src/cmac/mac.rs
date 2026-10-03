use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{self, Display, Formatter};

use tc_block_cipher::{BlockCipher, BlockCipherInit, KeyParams};

use super::shared::CmacCore;
use crate::{Mac, MacError, MacInit, MacInitError};

pub struct Cmac<C> {
    core: CmacCore<C, Vec<u8>>,
}

impl<C: BlockCipher> Cmac<C> {
    /// tag 預設一整個 block，同 BC。block 不是 8 或 16 bytes 時 panic。
    pub fn new(cipher: C) -> Self {
        let mac_size = cipher.block_size();
        Self::with_mac_size(cipher, mac_size)
    }

    /// `mac_size` 以 byte 計；不在 `1..=block size`，或 block 不是 8 或 16 bytes 時 panic。
    pub fn with_mac_size(cipher: C, mac_size: usize) -> Self {
        let block_size = cipher.block_size();
        let buffers = core::array::from_fn(|_| vec![0; block_size]);
        Self {
            core: CmacCore::new(cipher, buffers, mac_size),
        }
    }
}

impl<C: Display> Display for Cmac<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // BC 回報內部 CBC 模式的名稱，會跟 CBC-MAC 撞名；這裡改寫 "/CMAC"
        write!(f, "{}/CMAC", self.core.cipher())
    }
}

impl<C> Mac for Cmac<C>
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

impl<C, P> MacInit<P> for Cmac<C>
where
    C: BlockCipher + BlockCipherInit<P>,
    P: KeyParams + ?Sized,
    <C as BlockCipherInit<P>>::Error: 'static,
{
    type Error = MacInitError<<C as BlockCipherInit<P>>::Error>;

    fn init(&mut self, params: &P) -> Result<(), Self::Error> {
        self.core.init(params)
    }
}
