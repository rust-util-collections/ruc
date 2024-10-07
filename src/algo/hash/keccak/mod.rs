use sha3::{Digest, Keccak256};

/// The `Keccak` hash output type.
pub type KeccakHash = [u8; 32];

pub fn hash_msg(msg: &[&[u8]]) -> KeccakHash {
    let mut hasher = Keccak256::new();

    msg.iter().for_each(|i| {
        hasher.update(i);
    });
    *hasher.finalize().as_ref()
}

#[inline(always)]
pub fn hash(i: &[u8]) -> KeccakHash {
    hash_msg(&[i])
}
