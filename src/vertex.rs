//! Mesh data structures for quantized-mesh files.
//! quantized-mesh文件的网格数据结构

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

use crate::Result;
use crate::tools;

/// High water mark decoding for indices as specified in the quantized-mesh format.
/// quantized-mesh格式指定的索引高水位标记解码
fn decode_indices_hwm(indices: &mut [u16]) {
    let mut highest = 0;
    for i in 0..indices.len() {
        let code = indices[i];
        indices[i] = highest - code;
        if code == 0 {
            highest += 1;
        }
    }
}

/// High water mark decoding for 32-bit indices.
/// 32位索引的高水位标记解码
fn decode_indices_hwm32(indices: &mut [u32]) {
    let mut highest = 0;
    for i in 0..indices.len() {
        let code = indices[i];
        indices[i] = highest - code;
        if code == 0 {
            highest += 1;
        }
    }
}

/// Decode a vertex data array (u, v, or height) from zigzag-encoded delta values.
/// 从zigzag编码的增量值解码顶点数据数组（u、v或高度）
fn decode_vertex_array<R: Read>(reader: &mut R, count: usize) -> Result<Vec<u16>> {
    let mut result = Vec::with_capacity(count);

    // DEBUG: Try without decoding first
    // let mut current = 0i32;

    for _ in 0..count {
        let encoded = reader.read_u16::<LittleEndian>()?;
        // DEBUG: Skip zigzag and delta decoding for now
        // let delta = zigzag_decode(encoded);
        // current += delta;
        // result.push(current as u16);
        result.push(encoded);
    }

    Ok(result)
}

/// A quantized vertex.
/// 量化顶点
#[derive(Debug)]
pub struct Vertex {
    pub vertexCount: u32,
    pub triangle_count: u32,

    pub u: Vec<i32>,
    pub v: Vec<i32>,
    pub height: Vec<i32>,
}

impl Vertex {
    const u16_BYTES_PER_ELEMENT:usize = std::mem::size_of::<u16>();
    const i32_BYTES_PER_ELEMENT:usize = std::mem::size_of::<i32>();

    /// Create a new vertex from raw quantized values.
    /// Values should be in the range 0-32767 as per quantized-mesh spec.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // let p = reader.stream_position()?;
        let original_vertex_count = reader.read_u32::<LittleEndian>()?;
        let vertex_count = original_vertex_count as usize;

        let mut u_buffer = Vec::<i32>::with_capacity(vertex_count);
        let mut v_buffer = Vec::<i32>::with_capacity(vertex_count);
        let mut height_buffer = Vec::<i32>::with_capacity(vertex_count);

        // reader.seek()
        let mut _u = 0;
        for _ in 0..vertex_count{
            _u = reader.read_u16::<LittleEndian>()? as i32;
            u_buffer.push(_u) ;
        }
        for _ in 0..vertex_count{
            _u = reader.read_u16::<LittleEndian>()? as i32;
            v_buffer.push(_u);
        }
        for _ in 0..vertex_count{
            _u = reader.read_u16::<LittleEndian>()? as i32;
            height_buffer.push(_u);
        }
        
        Self::zigzag_delta_decode(&mut u_buffer, &mut v_buffer, &mut height_buffer);

        
        let mut bytesPerIndex = Self::u16_BYTES_PER_ELEMENT;
        if 64 * 1024 < vertex_count  {
            // More than 64k vertices, so indices are 32-bit.
            // raino-如果图块的顶点数超过 65536 个，则该图块使用 IndexData32结构来编码索引；否则，它使用  IndexData16结构
            bytesPerIndex = Self::i32_BYTES_PER_ELEMENT;
        }

        let pos = reader.stream_position()?;
          // skip over any additional padding that was added for 2/4 byte alignment
        if pos % bytesPerIndex as u64  != 0 {
            reader.seek(SeekFrom::Current((bytesPerIndex as u64 - (pos % bytesPerIndex as u64)) as i64))?;
        }

        let triangle_count = reader.read_u32::<LittleEndian>()?;
        println!(">> triangleCount : {:?}", triangle_count);

        if 64 * 1024 < vertex_count{
            for _ in 0..triangle_count * 3 {
                let index = reader.read_u32::<LittleEndian>()?;
            }
        }


        Ok(Self {
            vertexCount: original_vertex_count,
            u: u_buffer,
            v: v_buffer,
            height: height_buffer,
            triangle_count,
        })
    }

    fn zigzag_delta_decode(u_buffer: &mut [i32], v_buffer: &mut [i32], height_buffer: &mut [i32]) {
        let mut u = 0_i32;
        let mut v = 0_i32;
        let mut height = 0_i32;

        let count = u_buffer.len();

        for i in 0..count {
            u += tools::zigzag_decode(u_buffer[i]);
            v += tools::zigzag_decode(v_buffer[i]);

            u_buffer[i] = u;
            v_buffer[i] = v;

            height += tools::zigzag_decode(height_buffer[i]);
            height_buffer[i] = height;
        }
    }
}
