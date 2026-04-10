//! Mesh data structures for quantized-mesh files.
//! quantized-mesh文件的网格数据结构

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

use crate::Result;
use crate::tools;
use crate::extention::QuantizedMeshExtensionIds;

/// A quantized vertex.
/// 量化顶点
#[derive(Debug)]
pub struct Vertex {
    pub vertex_count: usize,
    pub triangle_count: usize,

    pub u: Vec<i32>,
    pub v: Vec<i32>,
    pub height: Vec<i32>,

    // 索引指定顶点如何连接成三角形，少于65536个顶点使用u16 存储索引数据；多于65536个顶点使用u32 存储索引数据；统一使用u32存储
    // indexes_u16: Vec<u16>, //
    pub indexes: Vec<u32>, //
    /// 这些索引列表列出了图块边缘上的顶点。了解哪些顶点位于边缘有助于添加裙边来隐藏相邻细节层之间的缝隙
    pub west_indices: Vec<u32>,
    pub south_indices: Vec<u32>,
    pub east_indices: Vec<u32>,
    pub north_indices: Vec<u32>,
}

impl Vertex {
    const U16_BYTES_PER_ELEMENT: usize = std::mem::size_of::<u16>();
    const I32_BYTES_PER_ELEMENT: usize = std::mem::size_of::<i32>();

    /// Create a new vertex from raw quantized values.
    /// Values should be in the range 0-32767 as per quantized-mesh spec.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let p = reader.stream_position()?;
        if p == 0 {
            let _ = reader.seek(SeekFrom::Start(88))?;
        }
        let vertex_count = reader.read_u32::<LittleEndian>()? as usize;

        let mut u_buffer = Vec::<i32>::with_capacity(vertex_count);
        let mut v_buffer = Vec::<i32>::with_capacity(vertex_count);
        let mut height_buffer = Vec::<i32>::with_capacity(vertex_count);

        // reader.seek()
        let mut _u = 0;
        for _ in 0..vertex_count {
            _u = reader.read_u16::<LittleEndian>()? as i32;
            u_buffer.push(_u);
        }
        for _ in 0..vertex_count {
            _u = reader.read_u16::<LittleEndian>()? as i32;
            v_buffer.push(_u);
        }
        for _ in 0..vertex_count {
            _u = reader.read_u16::<LittleEndian>()? as i32;
            height_buffer.push(_u);
        }

        Self::zigzag_delta_decode(&mut u_buffer, &mut v_buffer, &mut height_buffer);

        let mut bytes_per_index = Self::U16_BYTES_PER_ELEMENT;
        if 64 * 1024 < vertex_count {
            // More than 64k vertices, so indices are 32-bit.
            // raino-如果图块的顶点数超过 65536 个，则该图块使用 IndexData32结构来编码索引；否则，它使用  IndexData16结构
            bytes_per_index = Self::I32_BYTES_PER_ELEMENT;
        }

        let pos = reader.stream_position()?;
        // skip over any additional padding that was added for 2/4 byte alignment
        if pos % bytes_per_index as u64 != 0 {
            reader.seek(SeekFrom::Current(
                (bytes_per_index as u64 - (pos % bytes_per_index as u64)) as i64,
            ))?;
        }

        let triangle_count = reader.read_u32::<LittleEndian>()? as usize;
        println!(">> triangleCount : {:?}", triangle_count);

        let mut indexes = Vec::<u32>::with_capacity(triangle_count as usize * 3);
        let _ = Self::parse_indexes(reader, vertex_count, triangle_count as usize, &mut indexes);

        let west_vertex_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut west_indices = Vec::<u32>::with_capacity(west_vertex_count);
        let _ =
            Self::parse_edge_indices(reader, vertex_count, west_vertex_count, &mut west_indices);

        let south_vertex_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut south_indices = Vec::<u32>::with_capacity(west_vertex_count);
        let _ =
            Self::parse_edge_indices(reader, vertex_count, south_vertex_count, &mut south_indices);

        let east_vertex_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut east_indices = Vec::<u32>::with_capacity(west_vertex_count);
        let _ =
            Self::parse_edge_indices(reader, vertex_count, east_vertex_count, &mut east_indices);

        let north_vertex_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut north_indices = Vec::<u32>::with_capacity(west_vertex_count);
        let _ =
            Self::parse_edge_indices(reader, vertex_count, north_vertex_count, &mut north_indices);

        Self::parse_extentions(reader, vertex_count)?;
        Ok(Self {
            vertex_count,
            u: u_buffer,
            v: v_buffer,
            height: height_buffer,
            triangle_count,
            indexes,
            west_indices,
            south_indices,
            east_indices,
            north_indices,
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

    fn parse_indexes<R: Read + Seek>(
        reader: &mut R,
        vertex_count: usize,
        triangle_count: usize,
        indexes: &mut Vec<u32>,
    ) -> Result<()> {
        if 64 * 1024 < vertex_count {
            for _ in 0..triangle_count * 3 {
                let index = reader.read_u32::<LittleEndian>()?;
                indexes.push(index);
            }
        } else {
            for _ in 0..triangle_count * 3 {
                let index = reader.read_u16::<LittleEndian>()? as u32;
                indexes.push(index);
            }
        }
        tools::decode_indices_hwm32(indexes);
        Ok(())
    }

    fn parse_edge_indices<R: Read + Seek>(
        reader: &mut R,
        vertex_count: usize,
        edge_indices_count: usize,
        indexes: &mut Vec<u32>,
    ) -> Result<()> {
        if 64 * 1024 < vertex_count {
            for _ in 0..edge_indices_count {
                let index = reader.read_u32::<LittleEndian>()?;
                indexes.push(index);
            }
        } else {
            for _ in 0..edge_indices_count {
                let index = reader.read_u16::<LittleEndian>()? as u32;
                indexes.push(index);
            }
        }
        Ok(())
    }

    fn parse_extentions<R: Read + Seek>(
        reader: &mut R,
        vertex_count: usize,
    ) -> Result<()> {
        let start_pos = reader.stream_position()?;
        // Get total file size
        let file_size = reader.seek(SeekFrom::End(0))?;
        // Restore position
        reader.seek(SeekFrom::Start(start_pos))?;

        while reader.stream_position()? < file_size {
            let extension_id = reader.read_u8()?;
            let extension_length = reader.read_u32::<LittleEndian>()? as u64;

            match extension_id {
                id if id == QuantizedMeshExtensionIds::OctVertexNormals as u8 => {
                    // OCT_VERTEX_NORMALS
                    // Expected length: vertex_count * 2
                    let expected_len = vertex_count * 2;
                    if extension_length != expected_len as u64 {
                        return Err(crate::Error::InvalidFormat(format!(
                            "OCT_VERTEX_NORMALS extension length mismatch: expected {}, got {}",
                            expected_len, extension_length
                        )));
                    }
                    // Skip normal data
                    reader.seek(SeekFrom::Current(extension_length as i64))?;
                }
                id if id == QuantizedMeshExtensionIds::WaterMask as u8 => {
                    // WATER_MASK
                    // Skip water mask data
                    reader.seek(SeekFrom::Current(extension_length as i64))?;
                }
                id if id == QuantizedMeshExtensionIds::Metadata as u8 => {
                    // METADATA
                    // Read string length
                    let string_length = reader.read_u32::<LittleEndian>()? as u64;
                    // Verify string length fits within extension_length
                    if string_length + 4 > extension_length {
                        return Err(crate::Error::InvalidFormat(format!(
                            "METADATA string length exceeds extension length: {} > {}",
                            string_length + 4, extension_length
                        )));
                    }
                    // Skip JSON string
                    reader.seek(SeekFrom::Current(string_length as i64))?;
                    // Skip any remaining bytes in this extension
                    let remaining = extension_length - (string_length + 4);
                    if remaining > 0 {
                        reader.seek(SeekFrom::Current(remaining as i64))?;
                    }
                }
                _ => {
                    // Unknown extension, skip it
                    reader.seek(SeekFrom::Current(extension_length as i64))?;
                }
            }
        }
        Ok(())
    }

    fn get_reader_size<R: Read + Seek>(reader: &mut R) -> Result<u64> {
        // 保存当前位置
        let current_pos = reader.stream_position()?;

        // 跳转到末尾获取总大小
        let size = reader.seek(SeekFrom::End(0))?;

        // 恢复原始位置
        reader.seek(SeekFrom::Start(current_pos))?;

        Ok(size)
    }
}
