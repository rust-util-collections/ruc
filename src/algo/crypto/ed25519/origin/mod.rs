use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::Rng;

pub(super) fn create_keypair() -> (SigningKey, VerifyingKey) {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    let sk = SigningKey::from_bytes(&bytes);
    let vk = VerifyingKey::from(&sk);
    (sk, vk)
}
