use crate::ende::hex;
use rand::{RngCore, rng};

#[inline(always)]
pub fn rand_jwt() -> String {
    let mut data = [0u8; 32];
    rng().fill_bytes(&mut data);
    hex::encode(data)
}

#[inline(always)]
pub fn rand_jwt_n(n: usize) -> String {
    hex::encode(rand_data(n))
}

#[inline(always)]
pub fn rand_data(len: usize) -> Vec<u8> {
    let mut data = vec![0u8; len];
    rng().fill_bytes(&mut data);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_rand() {
        let jwt = rand_jwt();
        assert_eq!(jwt.len(), 64);

        let jwt = rand_jwt_n(193);
        assert_eq!(jwt.len(), 2 * 193);

        let data = rand_data(121);
        assert_eq!(data.len(), 121);
    }
}
