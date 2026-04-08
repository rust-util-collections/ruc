use crate::*;
use base64::prelude::{BASE64_STANDARD, Engine};

#[inline(always)]
pub fn encode<T: AsRef<[u8]>>(orig: T) -> String {
    BASE64_STANDARD.encode(orig)
}

#[inline(always)]
pub fn decode(encoded: &str) -> Result<Vec<u8>> {
    decode_generic(encoded).c(d!())
}

#[inline(always)]
pub fn decode_generic<T: AsRef<[u8]>>(encoded: T) -> Result<Vec<u8>> {
    BASE64_STANDARD.decode(encoded).c(d!())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn roundtrip() {
        let msg = b"hello world \x00\xff binary data";
        let encoded = encode(msg);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn roundtrip_empty() {
        let encoded = encode(b"");
        let decoded = decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }
}
