use crate::*;
use std::io::Read;

/// Compress data using zstd at default compression level (3).
pub fn zstd_compress(inputs: &[u8]) -> Result<Vec<u8>> {
    zstd::encode_all(inputs, 0).c(d!())
}

/// Decompress zstd-compressed data.
///
/// NOTE: the output size is unbounded — a hostile "decompression bomb"
/// can exhaust memory. Use [`zstd_uncompress_bounded`] for untrusted input.
pub fn zstd_uncompress(inputs: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(inputs).c(d!())
}

/// Decompress zstd-compressed data,
/// erroring if the output exceeds `max_output_size` bytes.
pub fn zstd_uncompress_bounded(
    inputs: &[u8],
    max_output_size: usize,
) -> Result<Vec<u8>> {
    let mut de = zstd::Decoder::new(inputs)
        .c(d!())?
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
        let data = b"hello world, this is a test of zstd compression";
        let compressed = zstd_compress(data).unwrap();
        let uncompressed = zstd_uncompress(&compressed).unwrap();
        assert_eq!(&uncompressed, data);
    }

    #[test]
    fn t_compress_empty() {
        let data = b"";
        let compressed = zstd_compress(data).unwrap();
        let uncompressed = zstd_uncompress(&compressed).unwrap();
        assert_eq!(&uncompressed, data);
    }

    #[test]
    fn t_uncompress_invalid_input() {
        assert!(zstd_uncompress(b"definitely not zstd data").is_err());
    }

    #[test]
    fn t_uncompress_bounded() {
        let data = [7u8; 1024];
        let compressed = zstd_compress(&data).unwrap();
        assert_eq!(zstd_uncompress_bounded(&compressed, 1024).unwrap(), data);
        assert!(zstd_uncompress_bounded(&compressed, 1023).is_err());
    }

    #[test]
    #[cfg(feature = "algo_rand")]
    fn t_compress_random() {
        let data = crate::algo::rand::rand_data(4096);
        let compressed = zstd_compress(&data).unwrap();
        let uncompressed = zstd_uncompress(&compressed).unwrap();
        assert_eq!(uncompressed, data);
    }
}
