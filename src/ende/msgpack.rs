use crate::*;
use serde::{Deserialize, Serialize};

#[inline(always)]
pub fn encode<T>(t: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    rmp::to_vec(&t).c(d!())
}

#[inline(always)]
pub fn decode<T>(bytes: &[u8]) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    rmp::from_slice(bytes).c(d!())
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Sample {
        x: i32,
        y: String,
        z: Vec<u8>,
    }

    #[test]
    fn roundtrip() {
        let original = Sample {
            x: 42,
            y: "hello".to_owned(),
            z: vec![1, 2, 3],
        };
        let encoded = encode(&original).unwrap();
        let decoded: Sample = decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_primitives() {
        let encoded = encode(&"test string").unwrap();
        let decoded: String = decode(&encoded).unwrap();
        assert_eq!(decoded, "test string");
    }
}
