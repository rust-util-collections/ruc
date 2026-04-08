use crate::*;

/// Compress data using zstd at default compression level (3).
pub fn zstd_compress(inputs: &[u8]) -> Result<Vec<u8>> {
    zstd::encode_all(inputs, 0).c(d!())
}

/// Decompress zstd-compressed data.
pub fn zstd_uncompress(inputs: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(inputs).c(d!())
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
    #[cfg(feature = "algo_rand")]
    fn t_compress_random() {
        let data = crate::algo::rand::rand_data(4096);
        let compressed = zstd_compress(&data).unwrap();
        let uncompressed = zstd_uncompress(&compressed).unwrap();
        assert_eq!(uncompressed, data);
    }
}
