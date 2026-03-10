# Terrain Parser

A Rust library for parsing Cesium quantized-mesh terrain files.

## Overview

The quantized-mesh format is a compact binary format for terrain data used by Cesium. This library provides functionality to parse and work with these files.

## Features

- Parse quantized-mesh headers
- Extract vertex data (quantized positions)
- Parse triangle indices
- Handle edge indices for skirt vertices
- Calculate actual heights from normalized values

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
terrain_parser = "0.1"
```

### Example

```rust
use std::fs::File;
use std::io::BufReader;
use terrain_parser::Mesh;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("terrain.terrain")?;
    let mut reader = BufReader::new(file);

    let mut mesh = Mesh::parse(&mut reader)?;
    mesh.calculate_heights();

    println!("Vertices: {}", mesh.vertex_count());
    println!("Triangles: {}", mesh.triangle_count());
    println!("Center: {:?}", mesh.header.center());

    Ok(())
}
```

## File Format

The quantized-mesh format consists of:

1. **Header** - Contains metadata about the tile including center, height range, and bounding sphere
2. **Vertex Data** - Quantized vertex positions (u, v, height) as 16-bit integers
3. **Triangle Indices** - Triangle connectivity as 16-bit indices
4. **Edge Indices** - Optional edge indices for north, south, east, and west skirt vertices

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.