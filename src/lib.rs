//! A library for parsing Cesium quantized-mesh terrain files.
//! 用于解析Cesium quantized-mesh地形文件的库
//!
//! The quantized-mesh format is a compact binary format for terrain data used by Cesium.
//! quantized-mesh格式是Cesium使用的紧凑二进制地形数据格式
//! This library provides functionality to parse and work with these files.
//! 本库提供了解析和处理这些文件的功能

mod error;
mod extention;
mod header;
mod vertex;
pub mod tools;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

use serde::Serialize;
use std::io::{Read, Seek};

pub use error::Error;
pub use header::Header;
pub use vertex::Vertex;

/// Result type for the terrain parser library.
/// 地形解析器库的Result类型
pub type Result<T> = std::result::Result<T, Error>;


#[derive(Debug, Serialize)]
pub struct QuantizedMeshTerrain{
    pub header: Header,
    pub vertex: Vertex
}

/// Parse terrain data from raw bytes (e.g. from HTTP response or ArrayBuffer).
/// Auto-detects and decompresses gzip if the data starts with the gzip magic bytes.
pub fn parse_bytes(data: &[u8]) -> Result<QuantizedMeshTerrain> {
    let decompressed = match tools::decode_gzip(data) {
        Ok(d) => d,
        Err(_) => data.to_vec(),
    };
    let mut reader = std::io::Cursor::new(&decompressed);
    parse(&mut reader)
}


pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<QuantizedMeshTerrain>{
    let header = Header::parse(reader)?;
    let vertex = Vertex::parse(reader)?;
    Ok(QuantizedMeshTerrain{
        header,
        vertex
    })
}

