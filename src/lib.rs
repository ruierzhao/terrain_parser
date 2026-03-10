//! A library for parsing Cesium quantized-mesh terrain files.
//!
//! The quantized-mesh format is a compact binary format for terrain data used by Cesium.
//! This library provides functionality to parse and work with these files.

mod error;
mod header;
mod mesh;

pub use error::Error;
pub use header::Header;
pub use mesh::Mesh;

/// Result type for the terrain parser library.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        let vertex = mesh::Vertex::new(32767, 16383, 49151);
        assert!((vertex.u - 0.5).abs() < 0.0001);
        assert!((vertex.v - 0.25).abs() < 0.0001);
        assert!((vertex.height_normalized - 0.75).abs() < 0.0001);
        assert_eq!(vertex.raw, [32767, 16383, 49151]);
    }

    #[test]
    fn test_vertex_denormalize() {
        let vertex = mesh::Vertex::new(0, 65535, 32767);
        let (u, v, height) = vertex.denormalize(100.0, 200.0);
        assert!((u - 0.0).abs() < 0.0001);
        assert!((v - 1.0).abs() < 0.0001);
        assert!((height - 150.0).abs() < 0.0001); // 100 + 0.5 * (200-100)
    }
}
