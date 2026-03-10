//! Error types for the terrain parser.

use thiserror::Error;

/// Errors that can occur when parsing quantized-mesh files.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error while reading the file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The file format is invalid or corrupted.
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// The file header is invalid.
    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    /// Unsupported version of the quantized-mesh format.
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u32),

    /// The file is truncated or missing data.
    #[error("Truncated data: {0}")]
    TruncatedData(String),

    /// The bounding box in the header is invalid.
    #[error("Invalid bounding box: {0}")]
    InvalidBoundingBox(String),
}