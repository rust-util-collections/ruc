use crate::*;
use serde::{Deserialize, Serialize};

#[inline(always)]
pub fn json_encode<T>(t: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    serde_json::to_vec(&t).c(d!())
}

#[inline(always)]
pub fn json_encode_str<T>(t: &T) -> Result<String>
where
    T: Serialize,
{
    serde_json::to_string(&t).c(d!())
}

#[inline(always)]
pub fn json_decode<T>(bytes: &[u8]) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    serde_json::from_slice(bytes).c(d!())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn roundtrip() {
        let v = vec![1u32, 2, 3];
        let bytes = json_encode(&v).unwrap();
        let s = json_encode_str(&v).unwrap();
        assert_eq!(s.as_bytes(), bytes.as_slice());
        let back: Vec<u32> = json_decode(&bytes).unwrap();
        assert_eq!(back, v);
    }

    #[test]
    fn decode_invalid() {
        assert!(json_decode::<Vec<u32>>(b"{broken").is_err());
        assert!(json_decode::<Vec<u32>>(b"").is_err());
    }
}
