//! Example of parsing a quantized-mesh file.

use std::fs::File;
use std::io::BufReader;
use terrain_parser::Mesh;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if a file was provided as an argument
    let file_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --example parse_mesh <path_to_quantized_mesh_file>");
        std::process::exit(1);
    });

    // Open the file
    let file = File::open(&file_path)?;
    let mut reader = BufReader::new(file);

    // Parse the mesh
    let mut mesh = Mesh::parse(&mut reader)?;
    mesh.calculate_heights();

    // Print basic information
    println!("Successfully parsed: {}", file_path);
    println!("  Vertices: {}", mesh.vertex_count());
    println!("  Triangles: {}", mesh.triangle_count());
    println!("  Center: ({}, {}, {})",
        mesh.header.center_x,
        mesh.header.center_y,
        mesh.header.center_z);
    println!("  Height range: {} to {}",
        mesh.header.minimum_height,
        mesh.header.maximum_height);

    // Print first few vertices
    println!("\nFirst 5 vertices:");
    for (i, vertex) in mesh.vertices.iter().take(5).enumerate() {
        println!("  Vertex {}: u={:.4}, v={:.4}, height={:.2}",
            i, vertex.u, vertex.v, vertex.height);
    }

    // Print first few triangles
    println!("\nFirst 5 triangles:");
    for i in 0..5.min(mesh.triangle_count()) {
        let base = i * 3;
        println!("  Triangle {}: [{}, {}, {}]",
            i,
            mesh.indices[base],
            mesh.indices[base + 1],
            mesh.indices[base + 2]);
    }

    Ok(())
}