# Terrain Parser

Rust 库，用于解析 **Cesium quantized-mesh** 地形文件 —— CesiumJS 3D 地球可视化使用的一种紧凑二进制地形格式。

## 格式概览

quantized-mesh 文件的二进制布局：

| 区域 | 内容 |
|---|---|
| Header | 瓦片中心坐标 (f64×3)、高度范围 (f32×2)、包围球 (f64×4)、地平线遮挡点 (f64×3) |
| Vertex Data | 量化顶点 `[u16, u16, u16]` — u/v/height 三个通道，经 zigzag-delta 编码 |
| Index Data | 三角索引 (u16 或 u32，取决于顶点数是否超过 65536)，高水位标记编码 |
| Edge Indices | 四边裙边顶点索引 (north/south/east/west) |
| Extensions | 可选扩展：OctVertexNormals / WaterMask / Metadata (JSON) |

## 用法

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

## 命令行工具

`src/bin/terrain-parser.rs` 是一个 CLI 工具，可直接在终端解析 `.terrain` 文件并展示详细信息。支持自动检测并解压 gzip 压缩的地形文件。

```bash
# 人类可读的详细输出
cargo run --bin terrain-parser -- path/to/tile.terrain

# 自定义采样数量
cargo run --bin terrain-parser -- path/to/tile.terrain -v 20 -t 10

# JSON 格式输出（便于管道处理）
cargo run --bin terrain-parser -- path/to/tile.terrain --json > result.json
```

### 命令行参数

| 参数 | 说明 |
|---|---|
| `<file>` | 地形文件路径（必需） |
| `--json` | 输出 pretty-printed JSON 格式序列化的完整 Mesh 数据 |
| `-v, --sample-vertices <N>` | 人类可读输出中展示的采样顶点数（默认 5） |
| `-t, --sample-triangles <N>` | 人类可读输出中展示的采样三角形数（默认 5） |

人类可读输出包含 **Header**（中心坐标、高度范围、包围球、地平线遮挡点）、**Vertex Data**（顶点数、三角数、U/V/Height 范围、采样顶点和三角索引）、**Edge Indices**（四边裙边顶点计数及采样）、**Extensions**（法线、水掩码、元数据）等信息。JSON 模式输出完整的 `QuantizedMeshTerrain` 序列化结果。

## 从 HTTP 响应解析

地形文件通常通过网络获取（`reqwest` / `fetch`）。`parse_bytes` 接收字节切片，自动检测并解压 gzip：

```rust
use terrain_parser::parse_bytes;

// 使用 reqwest 获取地形数据
let response = reqwest::get("http://localhost:8000/data/terrain/5.terrain")
    .await?
    .bytes()
    .await?;
let mesh = parse_bytes(&response)?;

println!("Vertex count: {}", mesh.vertex.vertex_count);
```

在浏览器 WASM 中，JS 侧 `ArrayBuffer` 传入后直接调用 `parse_bytes` 即可，无需手动处理 `Cursor` 或 gzip。

## WebAssembly / 浏览器端

编译为 WASM 可在浏览器中直接解析地形文件：

### 构建

```bash
wasm-pack build --target web --out-dir examples/viewer/pkg
```

### 托管 WASM 并运行示例

由于浏览器安全策略，WASM 需要通过 HTTP 服务加载，不能从 `file://` 协议打开。

```bash
cd examples/viewer
npx serve .          # 访问 http://localhost:3000
# 或
python -m http.server 8000
```

打开浏览器后，点击 **"Load Sample"** 加载内置的 `5.terrain` 样本，或拖拽任意 `.terrain` 文件到页面上。解析结果按 **Header / Vertices / Indexes / Edge Indices / Extensions** 分标签展示，同时提供 **Raw JSON** 完整数据视图。

## 项目结构

```
src/
├── lib.rs                — 入口：QuantizedMeshTerrain + parse()
├── header.rs             — Header 结构体（小端序 f64/f32 读取）
├── vertex.rs             — Vertex 数据、三角索引、边缘索引、扩展解析
├── extention.rs          — 扩展：OctVertexNormals / WaterMask / Metadata
├── wasm.rs               — wasm-bindgen 导出接口
├── error.rs              — thiserror 错误枚举
├── tools.rs              — zigzag 解码、HWM 索引解码、gzip 解压
└── bin/
    └── terrain-parser.rs — CLI 工具（clap derive）
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
