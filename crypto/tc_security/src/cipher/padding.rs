#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Padding {
    NoPadding,
    Pkcs7,
    Iso10126,
    Iso7816,
    X923,
    Tbc,
    ZeroByte,
}
