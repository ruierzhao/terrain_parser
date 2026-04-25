//! Extension identifiers and data structures for quantized-mesh format.
//! quantized-mesh格式的扩展标识符和数据结构
//!
//! When using the Quantized-Mesh format, a tile may be returned that includes additional extensions,
//! such as PerVertexNormals, watermask, etc.
//! This module defines the unique identifiers for each type of extension data that has been appended
//! to the standard mesh data.
//!
//! @see CesiumTerrainProvider

use std::io::{Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;
use crate::Result;

/// Extension identifiers for quantized-mesh format
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizedMeshExtensionIds {
    /// Oct-Encoded Per-Vertex Normals are included as an extension to the tile mesh
    OctVertexNormals = 1,
    /// A watermask is included as an extension to the tile mesh
    WaterMask = 2,
    /// A json object contain metadata about the tile
    Metadata = 4,
}

/// Oct-encoded per-vertex normal data
/// Each vertex has 2 bytes for oct-encoded normal (xy components)
/// Structure: unsigned char xy[vertexCount * 2];
#[derive(Debug, Clone, Serialize)]
pub struct OctEncodedVertexNormals {
    /// Oct-encoded normal data for each vertex, length = vertex_count * 2
    /// Each vertex uses 2 bytes: xy components
    pub xy: Vec<u8>,
}

/// Water mask data
/// Fixed size 256x256 mask array
/// Structure: unsigned char mask[256 * 256];
#[derive(Debug, Clone, Serialize)]
pub struct WaterMask {
    /// Water mask data, fixed size 256x256 = 65536 bytes
    pub mask:  Vec<u8>,
}

/// Tile metadata as JSON string
#[derive(Debug, Clone, Serialize)]
pub struct Metadata {
    /// JSON string containing tile metadata
    pub json: String,
}

/// Container for all possible extensions
/// Each field is optional since extensions may not be present
#[derive(Debug, Clone, Default, Serialize)]
pub struct Extensions {
    /// Oct-encoded vertex normals, if present
    pub oct_vertex_normals: Option<OctEncodedVertexNormals>,
    /// Water mask data, if present
    pub water_mask: Option<WaterMask>,
    /// Tile metadata, if present
    pub metadata: Option<Metadata>,
    /// Unknown extensions (ID -> raw data)
    pub unknown: Vec<(u8, Vec<u8>)>,
}

impl Extensions {
    /// Parse extensions from the current position in the reader
    /// Assumes reader is positioned at the start of extensions section
    /// vertex_count is needed to validate OCT_VERTEX_NORMALS extension length
    pub fn parse<R: Read + Seek>(reader: &mut R, vertex_count: usize) -> Result<Self> {
        let start_pos = reader.stream_position()?;
        // Get total file size
        let file_size = reader.seek(SeekFrom::End(0))?;
        // Restore position
        reader.seek(SeekFrom::Start(start_pos))?;

        let mut extensions = Extensions::default();

        while reader.stream_position()? < file_size {
            let extension_id = reader.read_u8()?;
            println!(">> extension_id : {:?}", extension_id);
            let extension_length = reader.read_u32::<LittleEndian>()? as u64;
            println!(">> extension_length : {:?}", extension_length);

            match extension_id {
                id if id == QuantizedMeshExtensionIds::OctVertexNormals as u8 => {
                    // OCT_VERTEX_NORMALS
                    // Expected length: vertex_count * 2
                    let expected_len = vertex_count * 2;
                    // Read normal data
                    let mut xy = vec![0u8; expected_len as usize];
                    reader.read_exact(&mut xy)?;
                    extensions.oct_vertex_normals = Some(OctEncodedVertexNormals { xy });
                }
                id if id == QuantizedMeshExtensionIds::WaterMask as u8 => {
                    // WATER_MASK
                    // Expected length: 256 * 256 = 65536
                    const WATER_MASK_SIZE: u64 = 256 * 256;
                    if extension_length != WATER_MASK_SIZE {
                        return Err(crate::Error::InvalidFormat(format!(
                            "WATER_MASK extension length mismatch: expected {}, got {}",
                            WATER_MASK_SIZE, extension_length
                        )));
                    }
                    // Read water mask data
                    let mut mask = vec![0u8; extension_length as usize];
                    reader.read_exact(&mut mask)?;
                    extensions.water_mask = Some(WaterMask { mask });
                }
                id if id == QuantizedMeshExtensionIds::Metadata as u8 => {
                    // METADATA
                    // Read string length
                    let string_length = reader.read_u32::<LittleEndian>()? as usize;
                    println!(">> string_length : {:?}", string_length);
                    // Verify string length fits within extension_length
                    if string_length as u64 + 4 > extension_length {
                        return Err(crate::Error::InvalidFormat(format!(
                            "METADATA string length exceeds extension length: {} > {}",
                            string_length + 4, extension_length
                        )));
                    }
                    // Read JSON string
                    let mut json_bytes = vec![0u8; string_length];
                    reader.read_exact(&mut json_bytes)?;
                    let json = String::from_utf8(json_bytes)
                        .map_err(|e| crate::Error::InvalidFormat(format!("Invalid UTF-8 in metadata: {}", e)))?;
                    println!(">> json : {:?}", json);
                    extensions.metadata = Some(Metadata { json });
                    // Skip any remaining bytes in this extension
                    let remaining = extension_length - (string_length as u64 + 4);
                    if remaining > 0 {
                        reader.seek(SeekFrom::Current(remaining as i64))?;
                    }
                }
                _ => {
                    // Unknown extension, store raw data
                    let mut data = vec![0u8; extension_length as usize];
                    reader.read_exact(&mut data)?;
                    extensions.unknown.push((extension_id, data));
                }
            }
        }

        Ok(extensions)
    }

    /// Check if any extensions are present
    pub fn is_empty(&self) -> bool {
        self.oct_vertex_normals.is_none() &&
        self.water_mask.is_none() &&
        self.metadata.is_none() &&
        self.unknown.is_empty()
    }
}