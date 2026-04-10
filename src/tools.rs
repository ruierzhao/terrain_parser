/// Zigzag decode function as specified in the quantized-mesh format.
/// quantized-mesh格式指定的Zigzag解码函数
/// Decodes a 16-bit zigzag-encoded value to a signed 32-bit integer.
/// 将16位zigzag编码值解码为有符号32位整数
pub fn zigzag_decode(value: i32) -> i32 {
    ((value >> 1) as i32) ^ (-((value & 1) as i32))
}

/// Decode gzip-compressed data using the flate2 library.
/// 使用flate2库解码gzip压缩数据
/// # Arguments
/// * `data` - A byte slice containing gzip-compressed data
/// # Returns
/// * `Ok(Vec<u8>)` - Decompressed bytes
/// * `Err(Error)` - If decompression fails
pub fn decode_gzip(data: &[u8]) -> crate::Result<Vec<u8>> {
    use std::io::Read;
    use flate2::read::GzDecoder;
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)
        .map_err(|e| crate::Error::InvalidFormat(format!("Gzip decompression failed: {}", e)))?;
    Ok(decompressed)
}
