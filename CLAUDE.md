# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust library (`terrain_parser`) for parsing Cesium quantized-mesh terrain files. The quantized-mesh format is a compact binary format for terrain data used by Cesium for 3D globe visualization.

## Development Commands

### Building and Testing
- `cargo build` - Build the library
- `cargo test` - Run all tests (currently has floating-point precision issues in tests)
- `cargo test -- --nocapture` - Run tests with output displayed
- `cargo test test_vertex_creation` - Run a specific test
- `cargo check` - Check code for errors without building
- `cargo clippy` - Run Clippy linter (install with `rustup component add clippy`)
- `cargo fmt` - Format code (install with `rustup component add rustfmt`)

### Documentation and Examples
- `cargo doc --open` - Build and open documentation
- `cargo run --example parse_mesh <path_to_file>` - Run the example parser with a terrain file
- `cargo build --examples` - Build all examples

### Publishing
- `cargo publish --dry-run` - Test publishing without uploading
- `cargo publish` - Publish to crates.io (requires login)

## Architecture

### Core Data Structures
- `Mesh` (`src/mesh.rs`) - Main container for quantized mesh data including vertices, indices, and edge indices
- `Header` (`src/header.rs`) - File header containing metadata (center coordinates, height range, bounding sphere)
- `Vertex` (`src/mesh.rs`) - Quantized vertex with raw 16-bit values and normalized coordinates
- `Error` (`src/error.rs`) - Comprehensive error enum using `thiserror` crate

### Parsing Flow
1. `Mesh::parse()` reads from a `Read + Seek` source
2. Calls `Header::parse()` to read metadata (skips vertex count, reads doubles and floats in little-endian)
3. Reads vertex count and vertex data (u16 values normalized to 0.0-1.0 range)
4. Reads triangle indices (groups of 3 u16 values)
5. Reads optional edge indices for north, south, east, and west skirt vertices

### Key Dependencies
- `byteorder` - Binary parsing with explicit endianness
- `thiserror` - Ergonomic error types
- `log` - Logging facade (currently unused in implementation)
- `bytemuck` - Zero-copy type casting (currently unused in implementation)

### Design Patterns
- **Reader-based parsing** - All parsing methods accept `&mut R where R: Read + Seek`
- **Lazy height calculation** - Vertex heights are calculated separately via `calculate_heights()` using header's min/max
- **Normalized coordinates** - Raw u16 values (0-65535) are stored as normalized f32 (0.0-1.0)
- **Comprehensive error handling** - Specific error variants for different failure modes

## File Format Notes

The quantized-mesh format follows this binary layout:
1. `u32` - Vertex count
2. `Header` - Metadata (see `Header` struct)
3. Vertex array - Each vertex: `[u16, u16, u16]` for (u, v, height)
4. `u32` - Triangle count, followed by triangle indices (`u16` × 3 per triangle)
5. Four edge index sections (north, south, east, west), each with `u32` count then `u16` indices

## Development Notes

- The library is designed to parse existing quantized-mesh files, not generate them
- Floating-point comparisons in tests require epsilon tolerance due to f32 precision
- The `bytemuck` dependency is included but not yet used - consider adding `#[derive(Pod, Zeroable)]` for zero-copy operations
- Example usage is in `examples/parse_mesh.rs` which shows the complete parsing workflow
- No external quantized-mesh test files are included; users must provide their own terrain files