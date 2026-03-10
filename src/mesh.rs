//! Mesh data structures for quantized-mesh files.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

use crate::Result;

/// A quantized mesh containing vertices, indices, and edge indices.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// The header of the mesh.
    pub header: crate::Header,

    /// Vertex data (quantized positions).
    pub vertices: Vec<Vertex>,

    /// Triangle indices.
    pub indices: Vec<u16>,

    /// Edge indices for skirt vertices.
    pub edge_indices_north: Vec<u16>,
    pub edge_indices_south: Vec<u16>,
    pub edge_indices_east: Vec<u16>,
    pub edge_indices_west: Vec<u16>,
}

/// A quantized vertex.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// Raw quantized coordinates.
    pub raw: [u16; 3],

    /// Height in meters.
    pub height: f32,

    /// Normalized coordinates (0.0 to 1.0).
    pub u: f32,
    pub v: f32,
    pub height_normalized: f32,
}

impl Mesh {
    /// Parse a complete mesh from a reader.
    ///
    /// The reader must be positioned at the start of the file.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let header = crate::Header::parse(reader)?;

        // Parse vertex count and vertex data
        reader.seek(SeekFrom::Start(0))?;
        let vertex_count = reader.read_u32::<LittleEndian>()? as usize;

        let mut vertices = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            let u = reader.read_u16::<LittleEndian>()?;
            let v = reader.read_u16::<LittleEndian>()?;
            let height = reader.read_u16::<LittleEndian>()?;
            vertices.push(Vertex {
                raw: [u, v, height],
                height: 0.0, // Will be calculated later
                u: u as f32 / 65535.0,
                v: v as f32 / 65535.0,
                height_normalized: height as f32 / 65535.0,
            });
        }

        // Parse triangle indices
        let triangle_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut indices = Vec::with_capacity(triangle_count * 3);
        for _ in 0..triangle_count {
            indices.push(reader.read_u16::<LittleEndian>()?);
            indices.push(reader.read_u16::<LittleEndian>()?);
            indices.push(reader.read_u16::<LittleEndian>()?);
        }

        // Parse edge indices (optional sections)
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
    pub fn calculate_heights(&mut self) {
        let height_range = self.header.maximum_height - self.header.minimum_height;

        for vertex in &mut self.vertices {
            vertex.height = self.header.minimum_height + vertex.height_normalized * height_range;
        }
    }

    /// Get the number of triangles in the mesh.
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
    pub fn new(u: u16, v: u16, height: u16) -> Self {
        Self {
            raw: [u, v, height],
            height: 0.0,
            u: u as f32 / 65535.0,
            v: v as f32 / 65535.0,
            height_normalized: height as f32 / 65535.0,
        }
    }

    /// Get the denormalized coordinates.
    pub fn denormalize(&self, min_height: f32, max_height: f32) -> (f32, f32, f32) {
        let height = min_height + self.height_normalized * (max_height - min_height);
        (self.u, self.v, height)
    }
}