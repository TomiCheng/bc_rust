#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DigestAlgorithm {
    #[cfg(feature = "md2")]
    Md2,
    #[cfg(feature = "md4")]
    Md4,
    #[cfg(feature = "md5")]
    Md5,
    #[cfg(feature = "sha1")]
    Sha1,
    #[cfg(feature = "sha224")]
    Sha224,
    #[cfg(feature = "sha256")]
    Sha256,
    #[cfg(feature = "sha384")]
    Sha384,
    #[cfg(feature = "sha512")]
    Sha512,
    #[cfg(feature = "sha512-224")]
    Sha512_224,
    #[cfg(feature = "sha512-256")]
    Sha512_256,
}
