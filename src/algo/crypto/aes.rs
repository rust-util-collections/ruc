use crate::{algo::hash::keccak, ende::base64, *};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngExt;

const NONCE_SIZE: usize = 12;

pub fn encrypt(password: &str, contents: &[u8]) -> Result<Vec<u8>> {
    let key_bytes = keccak::hash(password.as_bytes());
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = Aes256Gcm::new((&key_bytes).into())
        .encrypt(nonce, contents)
        .map_err(|e| eg!(e))?;
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn encrypt_to_base64(password: &str, contents: &[u8]) -> Result<String> {
    encrypt(password, contents)
        .c(d!())
        .map(|en| base64::encode(&en))
}

pub fn decrypt(password: &str, encrypted_bytes: &[u8]) -> Result<Vec<u8>> {
    if encrypted_bytes.len() < NONCE_SIZE {
        return Err(eg!("ciphertext too short"));
    }
    let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    let key_bytes = keccak::hash(password.as_bytes());
    Aes256Gcm::new((&key_bytes).into())
        .decrypt(nonce, ciphertext)
        .map_err(|e| eg!(e))
}

pub fn decrypt_from_base64(
    password: &str,
    encrypted_base64: &str,
) -> Result<Vec<u8>> {
    base64::decode(encrypted_base64)
        .c(d!())
        .and_then(|en| decrypt(password, &en).c(d!()))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ende::base64;

    #[test]
    fn t_aes_gcm() {
        for i in 11_u128..1111 {
            let password = i.to_string();
            let contents = base64::encode(keccak::hash(password.as_bytes()))
                .as_bytes()
                .to_vec();
            let encrypted_bytes = encrypt(&password, &contents).unwrap();
            // encrypted bytes should be nonce + ciphertext
            assert!(encrypted_bytes.len() > NONCE_SIZE);
            let decrypted_bytes =
                decrypt(&password, &encrypted_bytes).unwrap();
            assert_eq!(contents, decrypted_bytes);
        }
    }

    #[test]
    fn t_aes_gcm_base64() {
        for i in -1111_i128..-11 {
            let password = i.to_string();
            let contents = base64::encode(keccak::hash(password.as_bytes()))
                .as_bytes()
                .to_vec();
            let encrypted_base64 =
                encrypt_to_base64(&password, &contents).unwrap();
            let decrypted_bytes =
                decrypt_from_base64(&password, &encrypted_base64).unwrap();
            assert_eq!(contents, decrypted_bytes);
        }
    }

    #[test]
    fn t_aes_gcm_different_nonces() {
        let password = "test_password";
        let contents = b"hello world";
        let enc1 = encrypt(password, contents).unwrap();
        let enc2 = encrypt(password, contents).unwrap();
        // Same plaintext should produce different ciphertexts due to random nonce
        assert_ne!(enc1, enc2);
        // Both should decrypt correctly
        assert_eq!(decrypt(password, &enc1).unwrap(), contents);
        assert_eq!(decrypt(password, &enc2).unwrap(), contents);
    }

    #[test]
    fn t_aes_gcm_short_input() {
        let password = "test";
        assert!(decrypt(password, &[0u8; 5]).is_err());
    }
}
