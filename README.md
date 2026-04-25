# Terrain Parser

A Rust library for parsing **Cesium quantized-mesh** terrain files — the compact binary terrain format used by CesiumJS for 3D globe visualization.

## Format Overview

Binary layout of a quantized-mesh file:

| Section | Content |
|---|---|
| Header | Tile center (f64×3), height range (f32×2), bounding sphere (f64×4), horizon occlusion point (f64×3) |
| Vertex Data | Quantized vertices `[u16, u16, u16]` — three channels (u/v/height), zigzag-delta encoded |
| Index Data | Triangle indices (u16 or u32 depending on whether vertex count exceeds 65536), high-water-mark encoded |
| Edge Indices | Four edge skirt vertex index lists (north/south/east/west) |
| Extensions | Optional: OctVertexNormals / WaterMask / Metadata (JSON) |

## Usage

```toml
[dependencies]
terrain_parser = "0.1"
```

```rust
use std::io::Cursor;
use terrain_parser::parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read("tile.terrain")?;
    let mut reader = Cursor::new(&data);
    let mesh = parse(&mut reader)?;

    println!("Vertices: {}", mesh.vertex.vertex_count);
    println!("Triangles: {}", mesh.vertex.triangle_count);
    println!("Center: {:?}", mesh.header.center());
    Ok(())
}
```

### Parsing from bytes

`parse_bytes` accepts a byte slice, auto-detects gzip compression, and decompresses transparently:

```rust
use terrain_parser::parse_bytes;

let data = std::fs::read("tile.terrain")?;
let mesh = parse_bytes(&data)?;
```

### Parsing from an HTTP response

Terrain files are often fetched over the network. Use `parse_bytes` with e.g. `reqwest`:

```rust
use terrain_parser::parse_bytes;

let response = reqwest::get("http://localhost:8000/data/terrain/5.terrain")
    .await?
    .bytes()
    .await?;
let mesh = parse_bytes(&response)?;

println!("Vertex count: {}", mesh.vertex.vertex_count);
```

## CLI Tool

`src/bin/terrain-parser.rs` is a CLI tool for inspecting `.terrain` files in the terminal. It auto-detects and decompresses gzip-compressed terrain files.

```bash
# Human-readable detailed output
cargo run --bin terrain-parser -- path/to/tile.terrain

# Custom sample counts
cargo run --bin terrain-parser -- path/to/tile.terrain -v 20 -t 10

# JSON output (for piping)
cargo run --bin terrain-parser -- path/to/tile.terrain --json > result.json
```

### CLI Arguments

| Argument | Description |
|---|---|
| `<file>` | Path to terrain file (required) |
| `--json` | Output pretty-printed JSON of the full `QuantizedMeshTerrain` |
| `-v, --sample-vertices <N>` | Number of sample vertices shown in human-readable output (default: 5) |
| `-t, --sample-triangles <N>` | Number of sample triangles shown in human-readable output (default: 5) |

Human-readable output includes **Header** (center coordinates, height range, bounding sphere, horizon occlusion point), **Vertex Data** (vertex/triangle counts, U/V/Height ranges, sampled vertices and triangles), **Edge Indices** (counts and samples for all four edges), and **Extensions** (normals, water mask, metadata).

## WebAssembly / Browser

Compile to WASM for in-browser terrain file parsing.

### Build

```bash
wasm-pack build --target web --out-dir examples/viewer/pkg
```

### Serve and run the viewer

Browser security policies require WASM to be served over HTTP, not opened from `file://`.

```bash
cd examples/viewer
npx serve .          # Visit http://localhost:3000
# or
python -m http.server 8000
```

Open the browser, click **"Load Sample"** to load the bundled `5.terrain` sample, or drag-and-drop any `.terrain` file. Parsed results are displayed across tabs: **Header / Vertices / Indexes / Edge Indices / Extensions**, with a **Raw JSON** view for the complete dataset.

In your own WASM app, call `parse_bytes` with a `&[u8]` from a JS `ArrayBuffer` — no manual `Cursor` or gzip handling needed.

## Project Structure

```
src/
├── lib.rs                — Entry point: QuantizedMeshTerrain + parse() / parse_bytes()
├── header.rs             — Header struct (little-endian f64/f32 reader)
├── vertex.rs             — Vertex data, triangle indices, edge indices, extension dispatch
├── extention.rs          — Extensions: OctVertexNormals / WaterMask / Metadata
├── wasm.rs               — wasm-bindgen exports
├── error.rs              — thiserror error enum
├── tools.rs              — zigzag decode, HWM index decode, gzip decompression
└── bin/
    └── terrain-parser.rs — CLI tool (clap derive)
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
