use sha3::{Digest, Keccak256};

/// The `Keccak` hash output type.
pub type KeccakHash = [u8; 32];

/// Hash multiple slices as one message,
/// same as hashing their concatenation.
pub fn hash_msg(msg: &[&[u8]]) -> KeccakHash {
    let mut hasher = Keccak256::new();

    msg.iter().for_each(|i| {
        hasher.update(i);
    });
    *hasher.finalize().as_ref()
}

/// Keccak-256 hash of the input.
#[inline(always)]
pub fn hash(i: &[u8]) -> KeccakHash {
    hash_msg(&[i])
}
