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
