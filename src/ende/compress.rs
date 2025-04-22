use crate::*;
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{Read, Write};

pub fn zlib_compress(inputs: &[u8]) -> Result<Vec<u8>> {
    let mut en = ZlibEncoder::new(Vec::new(), Compression::default());
    en.write_all(inputs)
        .c(d!())
        .and_then(|_| en.finish().c(d!()))
}

pub fn zlib_uncompress(inputs: &[u8]) -> Result<Vec<u8>> {
    let mut de = ZlibDecoder::new(inputs);
    let mut res = vec![];
    de.read_to_end(&mut res).c(d!()).map(|_| res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "algo_rand")]
    fn t_compress_uncompress() {
        let data = crate::algo::rand::rand_data(128);
        let compressed = zlib_compress(&data).unwrap();
        let uncompressed = zlib_uncompress(&compressed).unwrap();
        assert_eq!(uncompressed, data);
    }
}
