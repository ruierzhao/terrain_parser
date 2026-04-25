use std::io::Cursor;
use std::path::{Path, PathBuf};
use clap::Parser;
use terrain_parser::parse;

#[derive(Parser)]
#[command(
    name = "terrain-parser",
    version = env!("CARGO_PKG_VERSION"),
    about = "Parse and display Cesium quantized-mesh terrain files"
)]
struct Cli {
    /// Path to the quantized-mesh terrain file (.terrain)
    file: PathBuf,

    /// Output as pretty-printed JSON instead of human-readable text
    #[arg(long)]
    json: bool,

    /// Number of sample vertices to display (ignored with --json)
    #[arg(short = 'v', long, default_value = "5")]
    sample_vertices: usize,

    /// Number of sample triangles to display (ignored with --json)
    #[arg(short = 't', long, default_value = "5")]
    sample_triangles: usize,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(&cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (data, was_gzip) = load_terrain_data(&cli.file)?;
    let mut reader = Cursor::new(&data);
    let mesh = parse(&mut reader)?;

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&mesh)?);
    } else {
        display(&mesh, &cli.file, was_gzip, cli.sample_vertices, cli.sample_triangles);
    }
    Ok(())
}

fn load_terrain_data(path: &Path) -> Result<(Vec<u8>, bool), Box<dyn std::error::Error>> {
    let data = std::fs::read(path)?;
    let is_gzip = data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b;
    if is_gzip {
        let decompressed = terrain_parser::tools::decode_gzip(&data)?;
        Ok((decompressed, true))
    } else {
        Ok((data, false))
    }
}

fn display(
    mesh: &terrain_parser::QuantizedMeshTerrain,
    path: &Path,
    was_gzip: bool,
    sample_vertices: usize,
    sample_triangles: usize,
) {
    let h = &mesh.header;
    let v = &mesh.vertex;

    println!("File: {}", path.display());
    if was_gzip {
        println!("Compression: gzip (auto-decompressed)");
    }
    println!();

    // Header
    println!("── Header ──");
    println!("  Center (ECEF):    ({:.6}, {:.6}, {:.6})", h.center_x, h.center_y, h.center_z);
    println!("  Height range:     {:.3} .. {:.3}", h.minimum_height, h.maximum_height);
    println!("  Bounding sphere:  center ({:.6}, {:.6}, {:.6}), radius {:.6}",
        h.bounding_sphere_center_x, h.bounding_sphere_center_y,
        h.bounding_sphere_center_z, h.bounding_sphere_radius);
    println!("  Horizon point:    ({:.6}, {:.6}, {:.6})",
        h.horizon_occlusion_point_x, h.horizon_occlusion_point_y, h.horizon_occlusion_point_z);
    println!();

    // Vertex data
    println!("── Vertex Data ──");
    println!("  Vertex count:     {}", v.vertex_count);
    println!("  Triangle count:   {}", v.triangle_count);

    if v.vertex_count > 0 {
        let (u_min, u_max) = min_max(&v.u);
        let (v_min, v_max) = min_max(&v.v);
        let (h_min, h_max) = min_max(&v.height);
        println!("  U range:          {} .. {}", u_min, u_max);
        println!("  V range:          {} .. {}", v_min, v_max);
        println!("  Height range:     {} .. {}", h_min, h_max);

        let nv = sample_vertices.min(v.vertex_count);
        println!();
        println!("  First {} vertices:", nv);
        for i in 0..nv {
            println!("    [{}] u={}, v={}, height={}", i, v.u[i], v.v[i], v.height[i]);
        }
    }

    // Triangle samples
    if v.triangle_count > 0 {
        let nt = sample_triangles.min(v.triangle_count);
        println!();
        println!("  First {} triangles:", nt);
        for i in 0..nt {
            let base = i * 3;
            println!("    [{}] ({}, {}, {})", i, v.indexes[base], v.indexes[base + 1], v.indexes[base + 2]);
        }
    }
    println!();

    // Edge indices
    println!("── Edge Indices ──");
    let nv = sample_vertices;
    show_edge("West", &v.west_indices, nv);
    show_edge("South", &v.south_indices, nv);
    show_edge("East", &v.east_indices, nv);
    show_edge("North", &v.north_indices, nv);
    println!();

    // Extensions
    println!("── Extensions ──");
    match &v.extensions {
        None => println!("  (none)"),
        Some(ext) => {
            show_ext("OctVertexNormals", ext.oct_vertex_normals.as_ref().map(|n| n.xy.len()));
            show_ext("WaterMask", ext.water_mask.as_ref().map(|w| w.mask.len()));
            match &ext.metadata {
                Some(m) => println!("  Metadata:         {}", m.json),
                None => println!("  Metadata:         none"),
            }
            if ext.unknown.is_empty() {
                println!("  Unknown:          none");
            } else {
                for (id, data) in &ext.unknown {
                    println!("  Unknown[{}]:       {} bytes", id, data.len());
                }
            }
        }
    }
}

fn min_max(data: &[i32]) -> (i32, i32) {
    let min = data.iter().min().copied().unwrap_or(0);
    let max = data.iter().max().copied().unwrap_or(0);
    (min, max)
}

fn show_edge(name: &str, indices: &[u32], sample_count: usize) {
    print!("  {}: {} vertices", name, indices.len());
    if !indices.is_empty() {
        let n = sample_count.min(indices.len());
        print!("  [");
        for i in 0..n {
            if i > 0 { print!(", "); }
            print!("{}", indices[i]);
        }
        if indices.len() > n { print!(", ..."); }
        print!("]");
    }
    println!();
}

fn show_ext(name: &str, byte_len: Option<usize>) {
    match byte_len {
        Some(n) => println!("  {}: present ({} bytes)", name, n),
        None => println!("  {}: absent", name),
    }
}
