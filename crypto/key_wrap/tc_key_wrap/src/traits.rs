//! Key-wrapping contracts.

/// The operation selected during key-wrapper initialization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WrapDirection {
    /// Protect key material and produce a wrapped blob.
    Wrap,
    /// Recover and authenticate key material from a wrapped blob.
    Unwrap,
}

/// An initialized key-wrapping algorithm.
///
/// This trait uses caller-provided output buffers, so the contract itself
/// requires neither `alloc` nor `std`. Initialization is provided independently
/// by [`KeyWrapInit`].
///
/// Implementations with the same [`Error`](KeyWrap::Error) type can be stored
/// behind `dyn KeyWrap<Error = E>` after initialization.
pub trait KeyWrap {
    /// The failure type returned by sizing and key-wrapping operations.
    type Error: core::error::Error;

    /// Returns the exact output length required to wrap `input_len` bytes.
    ///
    /// Invalid input lengths and arithmetic overflow must be reported as an
    /// error rather than being deferred to [`wrap_into`](KeyWrap::wrap_into).
    fn wrapped_len(&self, input_len: usize) -> Result<usize, Self::Error>;

    /// Returns an output capacity sufficient to unwrap `input_len` bytes.
    ///
    /// Some formats encode the original key length inside the authenticated
    /// wrapped blob, so the exact length is unavailable before unwrapping. The
    /// successful return value from [`unwrap_into`](KeyWrap::unwrap_into)
    /// reports how many bytes were actually written.
    fn max_unwrapped_len(&self, input_len: usize) -> Result<usize, Self::Error>;

    /// Wraps `input` into `output` and returns the number of bytes written.
    ///
    /// `output` must have at least the capacity reported by
    /// [`wrapped_len`](KeyWrap::wrapped_len) for this input length.
    fn wrap_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error>;

    /// Unwraps and authenticates `input` into `output`, returning the number of
    /// recovered key bytes written.
    ///
    /// `output` must have at least the capacity reported by
    /// [`max_unwrapped_len`](KeyWrap::max_unwrapped_len). Implementations must
    /// not leave recovered, unauthenticated key material in `output` when an
    /// integrity check fails.
    fn unwrap_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error>;
}

/// Initializes a key wrapper from parameters of type `P`.
///
/// This trait is independent of [`KeyWrap`]. Consumers that need both
/// capabilities use `W: KeyWrap + KeyWrapInit<P>`. Keeping `P` as a trait
/// parameter lets one caller-owned parameter object flow through composing
/// cryptographic layers.
pub trait KeyWrapInit<P: ?Sized> {
    /// The failure type returned by initialization.
    type Error: core::error::Error;

    /// Initializes the implementation for wrapping or unwrapping.
    fn init(&mut self, direction: WrapDirection, params: &P) -> Result<(), Self::Error>;
}
