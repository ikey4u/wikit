//! Thin safe wrappers around `libzstd-rs-sys` for wikit dictionary payloads.

use crate::error::{WikitError, WikitResult};

use libzstd_rs_sys::{
    ZSTD_compress, ZSTD_compressBound, ZSTD_decompress, ZSTD_getErrorName, ZSTD_isError,
};

/// Default compression level for small per-entry payloads (legacy v2).
pub const DEFAULT_ENTRY_LEVEL: i32 = 3;
/// Higher level for large dictionary blocks (v3) — targets better ratio than MDX zlib/LZO.
pub const DEFAULT_BLOCK_LEVEL: i32 = 9;
/// Uncompressed logical bytes per v3 block before zstd.
pub const DEFAULT_BLOCK_UNCOMPRESSED: usize = 512 * 1024;

fn zstd_error_message(code: usize) -> String {
    let name = ZSTD_getErrorName(code);
    if name.is_null() {
        format!("zstd error code {code}")
    } else {
        unsafe { std::ffi::CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned()
    }
}

pub fn compress(src: &[u8], level: i32) -> WikitResult<Vec<u8>> {
    if src.is_empty() {
        return Ok(Vec::new());
    }
    let bound = ZSTD_compressBound(src.len());
    let mut dst = vec![0u8; bound];
    let written = unsafe {
        ZSTD_compress(
            dst.as_mut_ptr().cast(),
            dst.len(),
            src.as_ptr().cast(),
            src.len(),
            level,
        )
    };
    if ZSTD_isError(written) != 0 {
        return Err(WikitError::new(format!(
            "zstd compress failed: {}",
            zstd_error_message(written)
        )));
    }
    dst.truncate(written);
    Ok(dst)
}

pub fn decompress(src: &[u8], uncompressed_size: usize) -> WikitResult<Vec<u8>> {
    if uncompressed_size == 0 {
        return Ok(Vec::new());
    }
    if src.is_empty() {
        return Err(WikitError::new("zstd decompress got empty input"));
    }
    let mut dst = vec![0u8; uncompressed_size];
    let written = unsafe {
        ZSTD_decompress(
            dst.as_mut_ptr().cast(),
            dst.len(),
            src.as_ptr().cast(),
            src.len(),
        )
    };
    if ZSTD_isError(written) != 0 {
        return Err(WikitError::new(format!(
            "zstd decompress failed: {}",
            zstd_error_message(written)
        )));
    }
    if written != uncompressed_size {
        return Err(WikitError::new(format!(
            "zstd decompress size mismatch: expected {uncompressed_size}, got {written}"
        )));
    }
    Ok(dst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_text() {
        let src = b"<div class=\"entry\">hello hello hello dictionary</div>".repeat(8);
        let compressed = compress(&src, DEFAULT_ENTRY_LEVEL).unwrap();
        assert!(compressed.len() < src.len());
        let out = decompress(&compressed, src.len()).unwrap();
        assert_eq!(out, src);
    }
}
