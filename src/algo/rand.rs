use crate::ende::hex;
use rand::RngExt;

#[inline(always)]
pub fn rand_hex() -> String {
    let mut data = [0u8; 32];
    rand::rng().fill(&mut data);
    hex::encode(data)
}

#[inline(always)]
pub fn rand_hex_n(n: usize) -> String {
    hex::encode(rand_data(n))
}

#[inline(always)]
pub fn rand_data(len: usize) -> Vec<u8> {
    let mut data = vec![0u8; len];
    rand::rng().fill(&mut data[..]);
    data
}

/// Deprecated: use `rand_hex` instead
#[deprecated(since = "10.0.0", note = "renamed to `rand_hex`")]
#[inline(always)]
pub fn rand_jwt() -> String {
    rand_hex()
}

/// Deprecated: use `rand_hex_n` instead
#[deprecated(since = "10.0.0", note = "renamed to `rand_hex_n`")]
#[inline(always)]
pub fn rand_jwt_n(n: usize) -> String {
    rand_hex_n(n)
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
