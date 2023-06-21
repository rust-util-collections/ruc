#![allow(missing_docs)]

pub mod codec;
pub mod hasher;
pub mod sig;
pub mod trie;

pub use codec::base64::{
    decode as base64_decode, decode_generic as base64_decode_generic,
    encode as base64_encode,
};
pub use hasher::keccak::{
    hash as keccak_hash, hash_msg as keccak_hash_msg, KeccakHash,
};
pub use trie::trie_root;
