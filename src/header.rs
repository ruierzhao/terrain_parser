//! Header parsing for quantized-mesh files.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

use crate::Result;

/// The header of a quantized-mesh file.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// The center of the tile in Earth-centered Fixed coordinates.
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,

    /// The minimum and maximum heights in the tile.
    pub minimum_height: f32,
    pub maximum_height: f32,

    /// The bounding sphere center and radius.
    pub bounding_sphere_center_x: f64,
    pub bounding_sphere_center_y: f64,
    pub bounding_sphere_center_z: f64,
    pub bounding_sphere_radius: f64,

    /// The horizon occlusion point (optional).
    pub horizon_occlusion_point_x: f64,
    pub horizon_occlusion_point_y: f64,
    pub horizon_occlusion_point_z: f64,
}

impl Header {
    /// Parse a header from a reader.
    ///
    /// The reader must be positioned at the start of the header.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Skip the first 4 bytes (vertex count)
        reader.seek(SeekFrom::Current(4))?;

        let center_x = reader.read_f64::<LittleEndian>()?;
        let center_y = reader.read_f64::<LittleEndian>()?;
        let center_z = reader.read_f64::<LittleEndian>()?;

        let minimum_height = reader.read_f32::<LittleEndian>()?;
        let maximum_height = reader.read_f32::<LittleEndian>()?;

        let bounding_sphere_center_x = reader.read_f64::<LittleEndian>()?;
        let bounding_sphere_center_y = reader.read_f64::<LittleEndian>()?;
        let bounding_sphere_center_z = reader.read_f64::<LittleEndian>()?;
        let bounding_sphere_radius = reader.read_f64::<LittleEndian>()?;

        let horizon_occlusion_point_x = reader.read_f64::<LittleEndian>()?;
        let horizon_occlusion_point_y = reader.read_f64::<LittleEndian>()?;
        let horizon_occlusion_point_z = reader.read_f64::<LittleEndian>()?;

        Ok(Header {
            center_x,
            center_y,
            center_z,
            minimum_height,
            maximum_height,
            bounding_sphere_center_x,
            bounding_sphere_center_y,
            bounding_sphere_center_z,
            bounding_sphere_radius,
            horizon_occlusion_point_x,
            horizon_occlusion_point_y,
            horizon_occlusion_point_z,
        })
    }

    /// Get the center point as a tuple.
    pub fn center(&self) -> (f64, f64, f64) {
        (self.center_x, self.center_y, self.center_z)
    }

    /// Get the bounding sphere center as a tuple.
    pub fn bounding_sphere_center(&self) -> (f64, f64, f64) {
        (
            self.bounding_sphere_center_x,
            self.bounding_sphere_center_y,
            self.bounding_sphere_center_z,
        )
    }

    /// Get the horizon occlusion point as a tuple.
    pub fn horizon_occlusion_point(&self) -> (f64, f64, f64) {
        (
            self.horizon_occlusion_point_x,
            self.horizon_occlusion_point_y,
            self.horizon_occlusion_point_z,
        )
    }
}