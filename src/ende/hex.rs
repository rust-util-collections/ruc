use crate::*;

#[inline(always)]
pub fn encode<T: AsRef<[u8]>>(orig: T) -> String {
    hex::encode(orig)
}

#[inline(always)]
pub fn decode(encoded: &str) -> Result<Vec<u8>> {
    decode_generic(encoded).c(d!())
}

#[inline(always)]
pub fn decode_generic<T: AsRef<[u8]>>(encoded: T) -> Result<Vec<u8>> {
    hex::decode(encoded).c(d!())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ende() {
        let msg = "alajflajfljaljflajfaljlaksjr22142";
        let encoded = encode(msg);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded.as_slice(), msg.as_bytes());
    }
}
