/// Zigzag decode function as specified in the quantized-mesh format.
/// quantized-mesh格式指定的Zigzag解码函数
/// Decodes a 16-bit zigzag-encoded value to a signed 32-bit integer.
/// 将16位zigzag编码值解码为有符号32位整数
pub fn zigzag_decode(value: i32) -> i32 {
    ((value >> 1) as i32) ^ (-((value & 1) as i32))
}

/// High water mark decoding for 32-bit indices.
/// 32位索引的高水位标记解码
pub fn decode_indices_hwm32(indices: &mut [u32]) {
    let mut highest = 0;
    for i in 0..indices.len() {
        let code = indices[i];
        indices[i] = highest - code;
        if code == 0 {
            highest += 1;
        }
    }
}

/// Decode gzip-compressed data using the flate2 library.
/// 使用flate2库解码gzip压缩数据
/// # Arguments
/// * `data` - A byte slice containing gzip-compressed data
/// # Returns
/// * `Ok(Vec<u8>)` - Decompressed bytes
/// * `Err(Error)` - If decompression fails
pub fn decode_gzip(data: &[u8]) -> crate::Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| crate::Error::InvalidFormat(format!("Gzip decompression failed: {}", e)))?;
    Ok(decompressed)
}
