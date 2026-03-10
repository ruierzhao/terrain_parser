//! A library for parsing Cesium quantized-mesh terrain files.
//! 用于解析Cesium quantized-mesh地形文件的库
//!
//! The quantized-mesh format is a compact binary format for terrain data used by Cesium.
//! quantized-mesh格式是Cesium使用的紧凑二进制地形数据格式
//! This library provides functionality to parse and work with these files.
//! 本库提供了解析和处理这些文件的功能

mod error;
mod header;
mod mesh;

pub use error::Error;
pub use header::Header;
pub use mesh::Mesh;

/// Result type for the terrain parser library.
/// 地形解析器库的Result类型
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        // Values in range 0-32767 as per quantized-mesh spec
        let vertex = mesh::Vertex::new(32767, 16383, 24575);
        assert!((vertex.u - 1.0).abs() < 0.0001);
        assert!((vertex.v - 0.5).abs() < 0.0001);
        assert!((vertex.height_normalized - 0.75).abs() < 0.0001);
        assert_eq!(vertex.raw, [32767, 16383, 24575]);
    }

    #[test]
    fn test_vertex_denormalize() {
        let vertex = mesh::Vertex::new(0, 32767, 16383);
        let (u, v, height) = vertex.denormalize(100.0, 200.0);
        assert!((u - 0.0).abs() < 0.0001);
        assert!((v - 1.0).abs() < 0.0001);
        assert!((height - 150.0).abs() < 0.0001); // 100 + 0.5 * (200-100)
    }
}
