#![allow(missing_docs)]

use blake3::{Hasher, OUT_LEN};

pub const HASH_SIZ: usize = OUT_LEN;

pub type Hash = [u8; HASH_SIZ];

#[inline(always)]
pub fn hash(data_list: &[&[u8]]) -> Hash {
    let mut hasher = Hasher::new();
    for data in data_list {
        hasher.update(data);
    }
    hasher.finalize().into()
}

#[inline(always)]
pub fn hash_single(data: &[u8]) -> Hash {
    hash(&[data])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let msg = b";lajgja";
        let h1 = hash_single(msg);
        let h2 = hash(&[msg]);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), HASH_SIZ);
    }
}
