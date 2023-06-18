use crate::*;
use base64::prelude::{Engine, BASE64_STANDARD};

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
