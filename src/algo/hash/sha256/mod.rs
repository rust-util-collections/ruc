use sha2::{Digest, Sha256};

/// The SHA-256 hash output type.
pub type Sha256Hash = [u8; 32];

pub fn hash_msg(msg: &[&[u8]]) -> Sha256Hash {
    let mut hasher = Sha256::new();

    msg.iter().for_each(|i| {
        hasher.update(i);
    });
    *hasher.finalize().as_ref()
}

#[inline(always)]
pub fn hash(i: &[u8]) -> Sha256Hash {
    hash_msg(&[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_sha256_hash() {
        let h1 = hash(b"hello");
        let h2 = hash(b"hello");
        assert_eq!(h1, h2);

        let h3 = hash(b"world");
        assert_ne!(h1, h3);
    }

    #[test]
    fn t_sha256_hash_msg() {
        let h1 = hash_msg(&[b"hello", b"world"]);
        let h2 = hash(b"helloworld");
        assert_eq!(h1, h2);
    }

    #[test]
    fn t_sha256_known_vector() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let h = hash(b"");
        let hex = h.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        assert_eq!(
            hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
