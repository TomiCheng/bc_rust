//! env 開關第一次用到時讀一次就固定，所以獨立成一個測試執行檔，且只放一個測試。

#![cfg(all(feature = "md5", feature = "sha1", feature = "sha256"))]

use tc_security::digest::{self, Algorithm};

#[test]
fn digests_named_in_the_env_switch_leave_the_list_but_can_still_be_created() {
    // SAFETY: 這個執行檔只有這一個測試，設定時沒有其他執行緒在讀環境變數
    unsafe { std::env::set_var("TC_SECURITY_DISABLED_DIGESTS", "md5, SHA-1") };

    let listed: Vec<_> = digest::algorithms()
        .map(|entry| entry.algorithm())
        .collect();
    assert!(!listed.contains(&Algorithm::Md5));
    assert!(!listed.contains(&Algorithm::Sha1));
    assert!(listed.contains(&Algorithm::Sha256));

    // 程式明確指定的演算法照樣建立
    assert_eq!(
        digest::get_by_name("MD5").map(|found| found.algorithm()),
        Ok(Algorithm::Md5)
    );
    assert_eq!(digest::get(Algorithm::Sha1).algorithm(), Algorithm::Sha1);
}
