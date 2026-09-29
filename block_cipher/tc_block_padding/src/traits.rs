pub trait BlockCipherPadding {
    /// The failure type returned by padding operations.
    type Error: core::error::Error;

    /// Pads `block[position..]` and returns the number of padding bytes added.
    ///
    /// `block` is one complete cipher block whose first `position` bytes hold
    /// the remaining message. Implementations overwrite every byte from
    /// `position` to the end of the block, so a `position` equal to the block
    /// length adds no bytes and leaves the block unchanged.
    ///
    /// The receiver is mutable because schemes that draw padding from a random
    /// generator advance that generator here.
    ///
    /// # Errors
    ///
    /// Returns an error when `position` is greater than the block length.
    fn add_padding(&mut self, block: &mut [u8], position: usize) -> Result<usize, Self::Error>;

    /// Returns the number of padding bytes at the end of `block`.
    ///
    /// The message occupies `block.len() - pad_count(block)` bytes. Callers
    /// must treat the result as untrusted length information until the message
    /// itself has been authenticated.
    ///
    /// # Errors
    ///
    /// Self-describing schemes return an error when the trailing bytes are not
    /// a valid encoding. Schemes that encode no length always succeed.
    fn pad_count(&self, block: &[u8]) -> Result<usize, Self::Error>;
}

pub trait BlockCipherPaddingInit<P> {
    /// The failure type returned by initialization.
    type Error: core::error::Error;

    /// Initializes the padding scheme with the supplied parameters.
    fn init(&mut self, params: P) -> Result<(), Self::Error>;
}