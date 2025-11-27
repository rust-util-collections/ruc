use ed25519_zebra::{SigningKey, VerificationKey};
use rand::rng;

pub(super) fn create_keypair() -> (SigningKey, VerificationKey) {
    let sk = SigningKey::new(rng());
    let vk = VerificationKey::from(&sk);
    (sk, vk)
}
