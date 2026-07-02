use crate::*;
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{Read, Write};

/// Compress data with zlib at the default compression level.
pub fn zlib_compress(inputs: &[u8]) -> Result<Vec<u8>> {
    let mut en = ZlibEncoder::new(Vec::new(), Compression::default());
    en.write_all(inputs)
        .c(d!())
        .and_then(|_| en.finish().c(d!()))
}

/// Decompress zlib-compressed data.
///
/// NOTE: the output size is unbounded — a hostile "decompression bomb"
/// can exhaust memory. Use [`zlib_uncompress_bounded`] for untrusted input.
pub fn zlib_uncompress(inputs: &[u8]) -> Result<Vec<u8>> {
    let mut de = ZlibDecoder::new(inputs);
    let mut res = vec![];
    de.read_to_end(&mut res).c(d!()).map(|_| res)
}

/// Decompress zlib-compressed data,
/// erroring if the output exceeds `max_output_size` bytes.
pub fn zlib_uncompress_bounded(
    inputs: &[u8],
    max_output_size: usize,
) -> Result<Vec<u8>> {
    let mut de = ZlibDecoder::new(inputs)
        .take((max_output_size as u64).saturating_add(1));
    let mut res = vec![];
    de.read_to_end(&mut res).c(d!())?;
    ensure!(
        res.len() <= max_output_size,
        "decompressed size exceeds the {max_output_size}-byte limit"
    );
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_compress_uncompress() {
        let data = b"an example, compress me please";
        let compressed = zlib_compress(data).unwrap();
        let uncompressed = zlib_uncompress(&compressed).unwrap();
        assert_eq!(&uncompressed, data);
    }

    #[test]
    fn t_compress_empty() {
        let data = b"";
        let compressed = zlib_compress(data).unwrap();
        let uncompressed = zlib_uncompress(&compressed).unwrap();
        assert_eq!(&uncompressed, data);
    }

    #[test]
    fn t_uncompress_invalid_input() {
        assert!(zlib_uncompress(b"definitely not zlib data").is_err());
    }

    #[test]
    fn t_uncompress_bounded() {
        let data = [7u8; 1024];
        let compressed = zlib_compress(&data).unwrap();
        assert_eq!(zlib_uncompress_bounded(&compressed, 1024).unwrap(), data);
        assert!(zlib_uncompress_bounded(&compressed, 1023).is_err());
    }

    #[test]
    #[cfg(feature = "algo_rand")]
    fn t_compress_random() {
        let data = crate::algo::rand::rand_data(128);
        let compressed = zlib_compress(&data).unwrap();
        let uncompressed = zlib_uncompress(&compressed).unwrap();
        assert_eq!(uncompressed, data);
    }
}
