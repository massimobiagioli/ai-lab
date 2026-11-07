//! Field encryption for secrets: AES-256-GCM with context-bound AAD.
//!
//! Ciphertext is bound to `{table}:{column}:{row_id}:{key_version}` so a value
//! cannot be moved to another row or column without failing authentication.

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

/// AES-GCM nonce length in bytes.
pub const NONCE_LEN: usize = 12;
/// AES-256 key length in bytes.
pub const KEY_LEN: usize = 32;

/// Encryption or key-decoding failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    /// The base64 key could not be decoded.
    InvalidKeyEncoding,
    /// The decoded key is not 32 bytes.
    InvalidKeyLength,
    /// Encryption failed.
    EncryptFailed,
    /// Decryption or authentication failed.
    DecryptFailed,
    /// The nonce is not 12 bytes.
    InvalidNonceLength,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidKeyEncoding => "invalid base64 key encoding",
            Self::InvalidKeyLength => "key must be 32 bytes",
            Self::EncryptFailed => "encryption failed",
            Self::DecryptFailed => "decryption or authentication failed",
            Self::InvalidNonceLength => "nonce must be 12 bytes",
        };
        f.write_str(message)
    }
}

impl std::error::Error for CryptoError {}

/// Encrypted payload with its nonce and key version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedSecret {
    /// Ciphertext bytes.
    pub ciphertext: Vec<u8>,
    /// 96-bit nonce.
    pub nonce: [u8; NONCE_LEN],
    /// Key version used for envelope rotation.
    pub key_version: i32,
}

/// A master key that encrypts and decrypts secret fields.
#[derive(Clone)]
pub struct MasterKey {
    cipher: Aes256Gcm,
    version: i32,
}

impl MasterKey {
    /// Decode a base64 32-byte key and build a cipher for `version`.
    pub fn from_base64(encoded: &str, version: i32) -> Result<Self, CryptoError> {
        let bytes = STANDARD
            .decode(encoded.trim())
            .map_err(|_| CryptoError::InvalidKeyEncoding)?;
        if bytes.len() != KEY_LEN {
            return Err(CryptoError::InvalidKeyLength);
        }
        let cipher =
            Aes256Gcm::new_from_slice(&bytes).map_err(|_| CryptoError::InvalidKeyLength)?;
        Ok(Self { cipher, version })
    }

    /// Key version used by this instance.
    #[must_use]
    pub const fn version(&self) -> i32 {
        self.version
    }

    /// Encrypt a plaintext with the provided AAD.
    pub fn encrypt(&self, plaintext: &[u8], aad: &str) -> Result<EncryptedSecret, CryptoError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| CryptoError::EncryptFailed)?;
        let nonce_bytes: [u8; NONCE_LEN] = nonce.into();
        Ok(EncryptedSecret {
            ciphertext,
            nonce: nonce_bytes,
            key_version: self.version,
        })
    }

    /// Decrypt a ciphertext with the provided AAD.
    pub fn decrypt(
        &self,
        ciphertext: &[u8],
        nonce: &[u8],
        aad: &str,
    ) -> Result<Vec<u8>, CryptoError> {
        if nonce.len() != NONCE_LEN {
            return Err(CryptoError::InvalidNonceLength);
        }
        self.cipher
            .decrypt(
                Nonce::from_slice(nonce),
                Payload {
                    msg: ciphertext,
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| CryptoError::DecryptFailed)
    }
}

/// Build the AAD that binds ciphertext to its location and key version.
#[must_use]
pub fn aad(table: &str, column: &str, row_id: &str, key_version: i32) -> String {
    format!("{table}:{column}:{row_id}:{key_version}")
}

/// Return the last four characters of a secret for display.
#[must_use]
pub fn last4(secret: &str) -> String {
    let chars: Vec<char> = secret.chars().collect();
    let start = chars.len().saturating_sub(4);
    chars[start..].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> MasterKey {
        let encoded = STANDARD.encode([7_u8; KEY_LEN]);
        MasterKey::from_base64(&encoded, 1).expect("valid key")
    }

    #[test]
    fn round_trip_returns_plaintext() {
        let key = key();
        let aad = aad("providers", "api_key", "row-1", 1);
        let encrypted = key.encrypt(b"super-secret", &aad).expect("encrypt");
        assert_eq!(encrypted.key_version, 1);
        assert_ne!(encrypted.ciphertext, b"super-secret");
        let plaintext = key
            .decrypt(&encrypted.ciphertext, &encrypted.nonce, &aad)
            .expect("decrypt");
        assert_eq!(plaintext, b"super-secret");
    }

    #[test]
    fn wrong_aad_fails_authentication() {
        let key = key();
        let encrypted = key
            .encrypt(b"super-secret", &aad("providers", "api_key", "row-1", 1))
            .expect("encrypt");
        let result = key.decrypt(
            &encrypted.ciphertext,
            &encrypted.nonce,
            &aad("providers", "api_key", "row-2", 1),
        );
        assert_eq!(result, Err(CryptoError::DecryptFailed));
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let key = key();
        let aad = aad("providers", "api_key", "row-1", 1);
        let mut encrypted = key.encrypt(b"super-secret", &aad).expect("encrypt");
        encrypted.ciphertext[0] ^= 0xff;
        assert_eq!(
            key.decrypt(&encrypted.ciphertext, &encrypted.nonce, &aad),
            Err(CryptoError::DecryptFailed)
        );
    }

    #[test]
    fn nonces_are_unique() {
        let key = key();
        let aad = aad("providers", "api_key", "row-1", 1);
        let first = key.encrypt(b"same", &aad).expect("encrypt");
        let second = key.encrypt(b"same", &aad).expect("encrypt");
        assert_ne!(first.nonce, second.nonce);
    }

    #[test]
    fn rejects_short_keys() {
        let encoded = STANDARD.encode([1_u8; 16]);
        assert_eq!(
            MasterKey::from_base64(&encoded, 1).err(),
            Some(CryptoError::InvalidKeyLength)
        );
    }

    #[test]
    fn last_four_characters() {
        assert_eq!(last4("sk-abcdef"), "cdef");
        assert_eq!(last4("abc"), "abc");
    }
}
