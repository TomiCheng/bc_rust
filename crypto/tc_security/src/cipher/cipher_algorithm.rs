#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Algorithm {
    #[cfg(feature = "aes")]
    Aes,
    #[cfg(feature = "aria")]
    Aria,
    #[cfg(feature = "rc2")]
    Rc2,
    #[cfg(feature = "rc5")]
    Rc5,
    #[cfg(feature = "rc5")]
    Rc5_64,
    #[cfg(feature = "rc6")]
    Rc6,
}
