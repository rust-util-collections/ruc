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
pub fn json_decode<T>(bytes: &[u8]) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    serde_json::from_slice(bytes).c(d!())
}
