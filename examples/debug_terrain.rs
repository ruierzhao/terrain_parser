//! Debug example to inspect terrain file structure and parse header

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "test_data/5.terrain";
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    // Read vertex count (first 4 bytes)
    let vertex_count_le = reader.read_u32::<LittleEndian>()?;
    let vertex_count_be = {
        reader.seek(SeekFrom::Start(0))?;
        reader.read_u32::<byteorder::BigEndian>()?
    };

    println!("=== File Analysis ===");
    println!("File: {}", file_path);
    println!("Vertex count (LE): {} (0x{:08x})", vertex_count_le, vertex_count_le);
    println!("Vertex count (BE): {} (0x{:08x})", vertex_count_be, vertex_count_be);

    // Check which vertex count looks more reasonable
    // For a 65KB file, reasonable vertex count is roughly:
    // (65536 - 4 - 88) / 6 ≈ 10833 vertices
    let file_size = reader.seek(SeekFrom::End(0))?;
    println!("File size: {} bytes", file_size);

    // Try to parse header assuming little-endian (as per spec)
    println!("\n=== Parsing Header (Little Endian) ===");
    reader.seek(SeekFrom::Start(4))?; // Skip vertex count

    let center_x = reader.read_f64::<LittleEndian>()?;
    let center_y = reader.read_f64::<LittleEndian>()?;
    let center_z = reader.read_f64::<LittleEndian>()?;

    let min_height = reader.read_f32::<LittleEndian>()?;
    let max_height = reader.read_f32::<LittleEndian>()?;

    let bs_center_x = reader.read_f64::<LittleEndian>()?;
    let bs_center_y = reader.read_f64::<LittleEndian>()?;
    let bs_center_z = reader.read_f64::<LittleEndian>()?;
    let bs_radius = reader.read_f64::<LittleEndian>()?;

    let hop_x = reader.read_f64::<LittleEndian>()?;
    let hop_y = reader.read_f64::<LittleEndian>()?;
    let hop_z = reader.read_f64::<LittleEndian>()?;

    println!("Center: ({}, {}, {})", center_x, center_y, center_z);
    println!("Height range: {} to {}", min_height, max_height);
    println!("Bounding sphere center: ({}, {}, {}), radius: {}", bs_center_x, bs_center_y, bs_center_z, bs_radius);
    println!("Horizon occlusion point: ({}, {}, {})", hop_x, hop_y, hop_z);

    // Check if values look reasonable
    // Earth-centered Fixed coordinates should be in meters from Earth's center
    // Earth radius is about 6,371,000 meters
    let earth_radius = 6_371_000.0;
    println!("\n=== Reasonableness Check ===");
    println!("Distance from origin: {} m", (center_x*center_x + center_y*center_y + center_z*center_z).sqrt());
    println!("Earth radius: {} m", earth_radius);

    // Try big-endian header
    println!("\n=== Parsing Header (Big Endian) ===");
    reader.seek(SeekFrom::Start(4))?;

    let center_x_be = reader.read_f64::<byteorder::BigEndian>()?;
    let center_y_be = reader.read_f64::<byteorder::BigEndian>()?;
    let center_z_be = reader.read_f64::<byteorder::BigEndian>()?;

    println!("Center (BE): ({}, {}, {})", center_x_be, center_y_be, center_z_be);
    println!("Distance from origin (BE): {} m", (center_x_be*center_x_be + center_y_be*center_y_be + center_z_be*center_z_be).sqrt());

    // Check first few bytes of vertex data
    println!("\n=== Vertex Data Start ===");
    let header_end = 4 + 88; // vertexCount + header
    reader.seek(SeekFrom::Start(header_end))?;

    let mut sample = [0u8; 12];
    reader.read_exact(&mut sample)?;
    println!("First 12 bytes after header: {:02x?}", sample);
    println!("As u16 (LE): {:?}", sample.chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect::<Vec<_>>());
    println!("As u16 (BE): {:?}", sample.chunks(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect::<Vec<_>>());

    Ok(())
}