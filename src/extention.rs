//! Extension identifiers for quantized-mesh format.
//! quantized-mesh格式的扩展标识符
//!
//! When using the Quantized-Mesh format, a tile may be returned that includes additional extensions,
//! such as PerVertexNormals, watermask, etc.
//! This enumeration defines the unique identifiers for each type of extension data that has been appended
//! to the standard mesh data.
//!
//! @see CesiumTerrainProvider

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
