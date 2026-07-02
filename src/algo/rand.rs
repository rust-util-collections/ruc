use crate::ende::hex;
use rand::RngExt;

/// Generate 32 random bytes, hex-encoded (64 chars).
#[inline(always)]
pub fn rand_hex() -> String {
    let mut data = [0u8; 32];
    rand::rng().fill(&mut data);
    hex::encode(data)
}

/// Generate `n` random bytes, hex-encoded (`2 * n` chars).
#[inline(always)]
pub fn rand_hex_n(n: usize) -> String {
    hex::encode(rand_data(n))
}

/// Generate `len` random bytes.
#[inline(always)]
pub fn rand_data(len: usize) -> Vec<u8> {
    let mut data = vec![0u8; len];
    rand::rng().fill(&mut data[..]);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_rand() {
        let token = rand_hex();
        assert_eq!(token.len(), 64);

        let token = rand_hex_n(193);
        assert_eq!(token.len(), 2 * 193);

        let data = rand_data(121);
        assert_eq!(data.len(), 121);
    }
}
