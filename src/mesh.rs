//! Mesh data structures for quantized-mesh files.
//! quantized-mesh文件的网格数据结构

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

use crate::Result;

/// Zigzag decode function as specified in the quantized-mesh format.
/// quantized-mesh格式指定的Zigzag解码函数
/// Decodes a 16-bit zigzag-encoded value to a signed 32-bit integer.
/// 将16位zigzag编码值解码为有符号32位整数
fn zigzag_decode(value: u16) -> i32 {
    ((value >> 1) as i32) ^ (-((value & 1) as i32))
}

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

/// A quantized mesh containing vertices, indices, and edge indices.
/// 包含顶点、索引和边缘索引的量化网格
#[derive(Debug, Clone)]
pub struct Mesh {
    /// The header of the mesh.
    /// 网格头部
    pub header: crate::Header,

    /// Vertex data (quantized positions).
    /// 顶点数据（量化位置）
    pub vertices: Vec<Vertex>,

    /// Triangle indices.
    /// 三角形索引
    pub indices: Vec<u16>,

    /// Edge indices for skirt vertices.
    /// 边缘（裙子）顶点索引
    pub edge_indices_north: Vec<u16>,
    pub edge_indices_south: Vec<u16>,
    pub edge_indices_east: Vec<u16>,
    pub edge_indices_west: Vec<u16>,
}

/// A quantized vertex.
/// 量化顶点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// Raw quantized coordinates.
    /// 原始量化坐标
    pub raw: [u16; 3],

    /// Height in meters.
    /// 高度（米）
    pub height: f32,

    /// Normalized coordinates (0.0 to 1.0).
    /// 标准化坐标（0.0到1.0）
    pub u: f32,
    pub v: f32,
    pub height_normalized: f32,
}

impl Mesh {
    /// Parse a complete mesh from a reader.
    /// 从读取器解析完整网格
    ///
    /// The reader must be positioned at the start of the file.
    /// 读取器必须位于文件起始位置
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Read vertex count (first 4 bytes)
        // 读取顶点数量（前4个字节）
        let original_vertex_count = reader.read_u32::<LittleEndian>()?;
        let mut vertex_count = original_vertex_count as usize;

        // Check if vertex count is reasonable
        // 检查顶点数量是否合理
        // If it's too large, try to infer from file size
        // 如果太大，尝试从文件大小推断
        if vertex_count > 1_000_000 {
            // Try to infer reasonable vertex count from file size
            let current_pos = reader.stream_position()?;
            let file_size = reader.seek(SeekFrom::End(0))?;
            reader.seek(SeekFrom::Start(current_pos))?;

            // File size minus vertex count (4) and header (88)
            // 文件大小减去顶点计数（4字节）和头部（88字节）
            if file_size > 92 {
                let available_bytes = file_size - 92;
                // Each vertex: 2 bytes for u + 2 for v + 2 for height = 6 bytes
                // 每个顶点：u 2字节 + v 2字节 + 高度 2字节 = 6字节
                let inferred_count = (available_bytes / 6) as usize;
                if inferred_count < 1_000_000 {
                    println!("Warning: Vertex count {} seems unreasonable, using inferred count {}",
                             vertex_count, inferred_count);
                    vertex_count = inferred_count;
                }
            }
        }

        // Parse header (Header::parse expects to be after vertex count)
        // 解析头部（Header::parse期望在顶点数量之后）
        let header = crate::Header::parse(reader)?;

        // Decode vertex arrays
        // 解码顶点数组
        let u_values = decode_vertex_array(reader, vertex_count)?;
        let v_values = decode_vertex_array(reader, vertex_count)?;
        let height_values = decode_vertex_array(reader, vertex_count)?;

        // Create vertices
        // 创建顶点
        let mut vertices = Vec::with_capacity(vertex_count);
        for i in 0..vertex_count {
            let u = u_values[i];
            let v = v_values[i];
            let height = height_values[i];
            vertices.push(Vertex {
                raw: [u, v, height],
                height: 0.0, // Will be calculated later // 稍后计算
                u: u as f32 / 32767.0,
                v: v as f32 / 32767.0,
                height_normalized: height as f32 / 32767.0,
            });
        }

        // Handle alignment padding before indices
        // 处理索引前的对齐填充
        // According to spec: ensure 2-byte alignment for 16-bit indices, 4-byte for 32-bit
        // 根据规范：确保16位索引2字节对齐，32位索引4字节对齐
        let pos = reader.stream_position()?;
        if vertex_count <= 65536 {
            // 16-bit indices, ensure 2-byte alignment
            if pos % 2 != 0 {
                reader.seek(SeekFrom::Current(1))?;
            }
        } else {
            // 32-bit indices, ensure 4-byte alignment
            let padding = (4 - (pos % 4)) % 4;
            if padding > 0 {
                reader.seek(SeekFrom::Current(padding as i64))?;
            }
        }

        // Parse triangle indices
        // 解析三角形索引
        let triangle_count = reader.read_u32::<LittleEndian>()? as usize;

        let mut indices = if vertex_count <= 65536 {
            // 16-bit indices
            let mut indices = Vec::with_capacity(triangle_count * 3);
            for _ in 0..triangle_count * 3 {
                indices.push(reader.read_u16::<LittleEndian>()?);
            }
            decode_indices_hwm(&mut indices);
            indices
        } else {
            // 32-bit indices
            let mut indices = Vec::with_capacity(triangle_count * 3);
            for _ in 0..triangle_count * 3 {
                indices.push(reader.read_u32::<LittleEndian>()?);
            }
            decode_indices_hwm32(&mut indices);
            // Convert to u16 if possible, but we'll store as Vec<u32> in a separate field
            // 如果可能转换为u16，但我们将存储在单独的Vec<u32>字段中
            // For now, return empty Vec<u16> as placeholder
            // 暂时返回空的Vec<u16>作为占位符
            Vec::new()
        };

        // Parse edge indices (optional sections)
        // 解析边缘索引（可选部分）
        // Note: Implementation incomplete - need to handle 16/32 bit based on vertex count
        // 注意：实现不完整 - 需要根据顶点数量处理16/32位
        let north_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut edge_indices_north = Vec::with_capacity(north_count);
        for _ in 0..north_count {
            edge_indices_north.push(reader.read_u16::<LittleEndian>()?);
        }

        let south_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut edge_indices_south = Vec::with_capacity(south_count);
        for _ in 0..south_count {
            edge_indices_south.push(reader.read_u16::<LittleEndian>()?);
        }

        let east_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut edge_indices_east = Vec::with_capacity(east_count);
        for _ in 0..east_count {
            edge_indices_east.push(reader.read_u16::<LittleEndian>()?);
        }

        let west_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut edge_indices_west = Vec::with_capacity(west_count);
        for _ in 0..west_count {
            edge_indices_west.push(reader.read_u16::<LittleEndian>()?);
        }

        Ok(Mesh {
            header,
            vertices,
            indices,
            edge_indices_north,
            edge_indices_south,
            edge_indices_east,
            edge_indices_west,
        })
    }

    /// Calculate actual heights for vertices using the header's min/max heights.
    /// 使用头部的最大最小高度计算顶点的实际高度
    pub fn calculate_heights(&mut self) {
        let height_range = self.header.maximum_height - self.header.minimum_height;

        for vertex in &mut self.vertices {
            vertex.height = self.header.minimum_height + vertex.height_normalized * height_range;
        }
    }

    /// Get the number of triangles in the mesh.
    /// 获取网格中的三角形数量
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Get the number of vertices in the mesh.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

impl Vertex {
    /// Create a new vertex from raw quantized values.
    /// Values should be in the range 0-32767 as per quantized-mesh spec.
    pub fn new(u: u16, v: u16, height: u16) -> Self {
        // According to spec, valid range is 0-32767
        let u_norm = u as f32 / 32767.0;
        let v_norm = v as f32 / 32767.0;
        let height_norm = height as f32 / 32767.0;

        Self {
            raw: [u, v, height],
            height: 0.0,
            u: u_norm,
            v: v_norm,
            height_normalized: height_norm,
        }
    }

    /// Get the denormalized coordinates.
    pub fn denormalize(&self, min_height: f32, max_height: f32) -> (f32, f32, f32) {
        let height = min_height + self.height_normalized * (max_height - min_height);
        (self.u, self.v, height)
    }
}