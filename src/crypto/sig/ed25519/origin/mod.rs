use ed25519_zebra::{SigningKey, VerificationKey};
use rand::thread_rng;

pub(super) fn create_keypair() -> (SigningKey, VerificationKey) {
    let sk = SigningKey::new(thread_rng());
    let vk = VerificationKey::from(&sk);
    (sk, vk)
}
