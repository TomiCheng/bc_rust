use tc_buffered_cipher::{BufferedCipher, BufferedCipherInit, CipherDirection};
use tc_zeroize::Zeroize;

use super::{AnyCipher, AnyError, AnyParams};

/// 用 `params` 初始化成加密後一次處理完 `input`，回傳新配置的密文。
pub fn encrypt(
    cipher: &mut AnyCipher,
    params: &AnyParams,
    input: &[u8],
) -> Result<Vec<u8>, AnyError> {
    process(cipher, CipherDirection::Encrypt, params, input)
}

/// 用 `params` 初始化成解密後一次處理完 `input`，回傳新配置的明文。
pub fn decrypt(
    cipher: &mut AnyCipher,
    params: &AnyParams,
    input: &[u8],
) -> Result<Vec<u8>, AnyError> {
    process(cipher, CipherDirection::Decrypt, params, input)
}

fn process(
    cipher: &mut AnyCipher,
    direction: CipherDirection,
    params: &AnyParams,
    input: &[u8],
) -> Result<Vec<u8>, AnyError> {
    cipher.init(direction, params)?;
    let mut output = vec![0; cipher.output_len(input.len())?];
    let result = cipher
        .process_bytes(input, &mut output)
        .and_then(|written| Ok(written + cipher.do_final(&mut output[written..])?));
    match result {
        Ok(written) => {
            output.truncate(written);
            Ok(output)
        }
        // 解密失敗時（例如 padding 或 tag 不對）可能已寫出部分明文，丟掉前先清除
        Err(error) => {
            output.zeroize();
            Err(error)
        }
    }
}
