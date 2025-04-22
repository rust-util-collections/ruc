use crate::{algo::hash::keccak, ende::base64, *};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};

pub fn encrypt(password: &str, contents: &[u8]) -> Result<Vec<u8>> {
    let key_bytes = keccak::hash(password.as_bytes());
    Aes256Gcm::new((&key_bytes).into())
        .encrypt(&Nonce::default(), contents)
        .map_err(|e| eg!(e))
}

pub fn encrypt_to_base64(password: &str, contents: &[u8]) -> Result<String> {
    encrypt(password, contents)
        .c(d!())
        .map(|en| base64::encode(&en))
}

pub fn decrypt(password: &str, encrypted_bytes: &[u8]) -> Result<Vec<u8>> {
    let key_bytes = keccak::hash(password.as_bytes());
    Aes256Gcm::new((&key_bytes).into())
        .decrypt(&Nonce::default(), encrypted_bytes)
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
    fn t_aec_gcm() {
        for i in 11_u128..1111 {
            let password = i.to_string();
            let contents = base64::encode(keccak::hash(password.as_bytes()))
                .as_bytes()
                .to_vec();
            let encrypted_bytes = encrypt(&password, &contents).unwrap();
            let decrypted_bytes =
                decrypt(&password, &encrypted_bytes).unwrap();
            assert_eq!(contents, decrypted_bytes);
        }
    }

    #[test]
    fn t_aec_gcm_base64() {
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
}
