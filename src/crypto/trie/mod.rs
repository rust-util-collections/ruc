use reference_trie::{calc_root, ExtensionLayout};
use std::fmt::Debug;

pub fn trie_root<I, A, B>(data: I) -> super::KeccakHash
where
    I: IntoIterator<Item = (A, B)>,
    A: AsRef<[u8]> + Ord + Debug,
    B: AsRef<[u8]> + Debug,
{
    calc_root::<ExtensionLayout, I, A, B>(data)
}

#[cfg(test)]
mod test {
    use super::super::*;
    use std::mem::size_of;

    #[test]
    fn basic() {
        let data = (0u32..1000)
            .map(|i| (keccak_hash(&to_bytes(i)), to_bytes(i)))
            .collect::<Vec<_>>();
        let hash = trie_root(data);

        let data1 = (0u32..1000)
            .map(|i| (to_bytes(i), to_bytes(i)))
            .collect::<Vec<_>>();
        let hash1 = trie_root(data1);

        assert!(hash != hash1);
    }

    fn to_bytes(i: u32) -> [u8; size_of::<u32>()] {
        u32::to_be_bytes(i)
    }
}
