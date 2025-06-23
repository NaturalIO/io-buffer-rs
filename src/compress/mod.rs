use std::io::Result;

/// A trait for different compress method
pub trait Compression {
    /// Estimate the upper bound of buffer size needed
    fn compress_bound(origin_len: usize) -> usize;

    /// On success, return the size of compressed data.
    ///
    /// Arguments:
    ///
    ///  * src: original data
    ///
    fn compress(src: &[u8], dest: &mut [u8]) -> Result<usize>;

    /// On success, return the size of decompressed data.
    ///
    /// Arguments:
    ///
    ///  * src: compressed data
    ///
    ///  * dest: output buffer for decompressed data
    fn decompress(src: &[u8], dest: &mut [u8]) -> Result<usize>;
}

#[cfg(any(feature = "lz4", doc))]
/// Enabled with feature `lz4`
pub mod lz4;
