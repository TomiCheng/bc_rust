use tc_block_modes::{CbcBlockCipher};

pub struct CbcMac<C> {
    cipher: CbcBlockCipher<C>
}

pub struct PaddedCbcMac<C, P> {
    mac: CbcMac<C>,
    padding: P
}