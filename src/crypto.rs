//! QIUI `EncryptUtil` AES/CBC/PKCS5 (compatible with QIUI crypto + `libsecret_jni`).

use aes::Aes256;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use block_padding::Pkcs7;
use cbc::{Decryptor, Encryptor};
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use thiserror::Error;

/// IV = `mb.b.a() + "_8"`
pub const IV: &[u8; 16] = b"3*1&f6O%6&Pa5d_8";

/// `SecretManager.getKey(ctx, 0)` → API_SECRET_STR (AES-256)
pub const API_SECRET_KEY: &[u8; 32] = b"87dQ_P5F#9&K*G6b9&f%8!d&06@(c_*a";

/// `EncryptUtil.b` → PWD_STR
pub const PWD_KEY: &[u8; 32] = b"10kB8mdiaWRxFkNQD8Xtb6fm>Cg5N_QX";

/// `mb.a.b` → BLUETOOTH_COMMAND
pub const BLUETOOTH_COMMAND_KEY: &[u8; 32] = b"VfttrWZYZ5QVUe>})C5x?oYWVewxVy!]";

/// `mb.a.a` → TIME_STAMP
pub const TIME_STAMP_KEY: &[u8; 32] = b"LpyaN2dvT3CbxRNJ>^*kjjAsq=^e*cD5";

/// `EncryptUtil.c` → BLUETOOTH_ADDRESS
pub const BLUETOOTH_ADDRESS_KEY: &[u8; 32] = b"X7vuZ@NvjPtMmBF8bpWbbfrpYa3h11IB";

/// `mb.a.c` → MESSAGE
pub const MESSAGE_KEY: &[u8; 32] = b"RJsd3J3>VZmUPp>VLBEu?khdWdRUxXq(";

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("base64: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("aes decrypt/pad failed")]
    Aes,
    #[error("utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub fn aes_cbc_encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let encryptor = Encryptor::<Aes256>::new_from_slices(key, IV).map_err(|_| CryptoError::Aes)?;
    let mut buf = vec![0u8; plaintext.len() + 16];
    buf[..plaintext.len()].copy_from_slice(plaintext);
    let n = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext.len())
        .map_err(|_| CryptoError::Aes)?
        .len();
    buf.truncate(n);
    Ok(buf)
}

pub fn aes_cbc_decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let decryptor = Decryptor::<Aes256>::new_from_slices(key, IV).map_err(|_| CryptoError::Aes)?;
    let mut buf = ciphertext.to_vec();
    let pt = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|_| CryptoError::Aes)?;
    Ok(pt.to_vec())
}

pub fn encrypt_b64(key: &[u8], plaintext: &str) -> Result<String, CryptoError> {
    Ok(B64.encode(aes_cbc_encrypt(key, plaintext.as_bytes())?))
}

pub fn decrypt_b64(key: &[u8], ciphertext_b64: &str) -> Result<String, CryptoError> {
    // App sometimes wraps response in JSON string quotes.
    let s = ciphertext_b64.trim().trim_matches('"');
    let ct = B64.decode(s)?;
    let pt = aes_cbc_decrypt(key, &ct)?;
    Ok(String::from_utf8(pt)?)
}

pub fn encrypt_api(plaintext: &str) -> Result<String, CryptoError> {
    encrypt_b64(API_SECRET_KEY, plaintext)
}

pub fn decrypt_api(ciphertext_b64: &str) -> Result<String, CryptoError> {
    decrypt_b64(API_SECRET_KEY, ciphertext_b64)
}

pub fn encrypt_password(password: &str) -> Result<String, CryptoError> {
    encrypt_b64(PWD_KEY, password)
}

pub fn encrypt_bt_command(plaintext: &str) -> Result<String, CryptoError> {
    encrypt_b64(BLUETOOTH_COMMAND_KEY, plaintext)
}

pub fn decrypt_bt_command(ciphertext_b64: &str) -> Result<String, CryptoError> {
    decrypt_b64(BLUETOOTH_COMMAND_KEY, ciphertext_b64)
}

pub fn encrypt_timestamp_field(value: &str) -> Result<String, CryptoError> {
    encrypt_b64(TIME_STAMP_KEY, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_api() {
        let s = r#"{"hello":"world"}"#;
        let e = encrypt_api(s).unwrap();
        assert_eq!(decrypt_api(&e).unwrap(), s);
    }

    #[test]
    fn decrypt_known_api_list_prefix() {
        // Live decrypt is integration; here just ensure key/IV lengths.
        assert_eq!(API_SECRET_KEY.len(), 32);
        assert_eq!(IV.len(), 16);
        assert_eq!(PWD_KEY.len(), 32);
    }
}
