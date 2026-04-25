//! WASM bindings for terrain-parser.
//! Exports parse functions that accept raw bytes and return JSON strings.

use std::io::Cursor;
use wasm_bindgen::prelude::*;

/// Parse raw terrain file bytes (possibly gzip-compressed) and return JSON.
/// Returns Ok(json_string) on success, Err(error_message) on failure.
#[wasm_bindgen]
pub fn parse_terrain_bytes(data: &[u8]) -> Result<String, JsValue> {
    // Try to decompress as gzip first; if that fails, use raw bytes
    let decompressed = match crate::tools::decode_gzip(data) {
        Ok(d) => d,
        Err(_) => data.to_vec(),
    };

    let mut reader = Cursor::new(&decompressed);
    let mesh = crate::parse(&mut reader)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    serde_json::to_string(&mesh)
        .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
}

/// Parse raw terrain file bytes, returning pretty-printed JSON.
#[wasm_bindgen]
pub fn parse_terrain_pretty(data: &[u8]) -> Result<String, JsValue> {
    let decompressed = match crate::tools::decode_gzip(data) {
        Ok(d) => d,
        Err(_) => data.to_vec(),
    };

    let mut reader = Cursor::new(&decompressed);
    let mesh = crate::parse(&mut reader)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    serde_json::to_string_pretty(&mesh)
        .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
}
