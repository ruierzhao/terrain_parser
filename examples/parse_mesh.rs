//! Example of parsing a quantized-mesh file.

use std::fs::File;
use std::io::{BufReader, Read, Cursor};
use byteorder::{LittleEndian,ReadBytesExt};
use terrain_parser::Vertex;
use terrain_parser::{parse, tools::decode_gzip};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if a file was provided as an argument
    let file_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: cargo run --example parse_mesh <path_to_quantized_mesh_file>");
        std::process::exit(1);
    });

    // Read the entire file
    let compressed_data = std::fs::read(&file_path)?;

    // Decompress gzip data
    // let decompressed_data = decode_gzip(&compressed_data)?;

    // Create a cursor over the decompressed data for parsing
    let mut reader = Cursor::new(compressed_data);
    // let mut reader = Cursor::new(decompressed_data);

    // let center_x = reader.read_f64::<LittleEndian>()?;
    
    // println!("center_x:{}",center_x);
    // Parse the mesh
    let mesh = parse(&mut reader)?;
    // let _ = Vertex::parse(&mut reader);

    Ok(())
}