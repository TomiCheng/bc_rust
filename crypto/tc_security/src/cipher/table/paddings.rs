use super::specs::PaddingSpec;
use crate::cipher::Padding;

/// 名稱同 BC，比對時不分大小寫。
pub(super) const PADDINGS: &[PaddingSpec] = &[
    PaddingSpec {
        padding: Padding::NoPadding,
        names: &["NOPADDING"],
    },
    PaddingSpec {
        padding: Padding::Pkcs7,
        names: &["PKCS7PADDING", "PKCS7", "PKCS5PADDING", "PKCS5"],
    },
    PaddingSpec {
        padding: Padding::Iso10126,
        names: &["ISO10126PADDING", "ISO10126D2PADDING", "ISO10126-2PADDING"],
    },
    PaddingSpec {
        padding: Padding::Iso7816,
        names: &["ISO7816-4PADDING", "ISO9797-1PADDING"],
    },
    PaddingSpec {
        padding: Padding::X923,
        names: &["X923PADDING"],
    },
    PaddingSpec {
        padding: Padding::Tbc,
        names: &["TBCPADDING"],
    },
    PaddingSpec {
        padding: Padding::ZeroByte,
        names: &["ZEROBYTEPADDING"],
    },
];
