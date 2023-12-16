use std::str::FromStr;

use bevy::{
    asset::{io::*, AssetLoader, ReflectAsset},
    math::{DVec2, DVec4},
    prelude::*,
    render::{
        extract_component::DynamicUniformIndex,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::Image,
    },
    utils::thiserror,
};

use crate::math::{
    coord::{GeoCoord2, GeoGridSpace, GeoIndex2, GridTransform, IndexTransform},
    DRect,
};

/// Ascii Grid Asset
#[derive(Reflect, Asset, Debug)]
#[reflect(Asset)]
pub struct AsciiGrid {
    /// (ncols, nrows)
    size: UVec2,
    ll_corner: DVec2, // lower left corner
    cell_size: f64,
    nodata_value: i16,
    space: GeoGridSpace,
    #[reflect(ignore)]
    data: Vec<i16>,
}

fn vec16_to_vec8(data: &[i16]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() * 2);
    for v in data {
        // TODO: what endianness does webgpu expect?
        result.extend_from_slice(&v.to_le_bytes());
    }
    result
}
fn vec16_to_rgb_vec8(data: &[i16]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() * 3);
    for v in data {
        // TODO: what endianness does webgpu expect?
        result.extend_from_slice(&[0, 256i16.saturating_div(*v) as u8, 0, 255]);
    }
    result
}
fn vec16_to_float_vec8(data: &[i16]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());
    for v in data {
        // TODO: what endianness does webgpu expect?
        result.extend_from_slice(&[0, 256i16.saturating_div(*v) as u8, 0, 255]);
    }
    result
}
fn vec16_to_rgba_vec8(data: &[i16]) -> Vec<u8> {
    data.iter().flat_map(|&v| v.to_le_bytes()).collect()
}
fn flat_map<U: IntoIterator<Item = u8>, F: FnMut(i16) -> U>(data: &[i16], mut f: F) -> Vec<u8> {
    data.iter().flat_map(|&v| f(v)).collect()
}

impl AsciiGrid {
    pub fn space(&self) -> &GeoGridSpace {
        &self.space
    }

    pub fn create_image(&self, format: TextureFormat) -> Image {
        let max_value = self.get_max_value().unwrap_or(0);
        let min_value: i16 = 0;
        let data = match format {
            TextureFormat::R16Sint => vec16_to_vec8(&self.data),
            TextureFormat::R16Snorm => vec16_to_vec8(&self.data),
            TextureFormat::R16Float => vec16_to_float_vec8(&self.data),
            TextureFormat::Rgba16Unorm => flat_map(&self.data, |v| {
                let mut d: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                if v == self.nodata_value {
                    return d;
                }
                let v = (v - min_value) as f64 / (max_value - min_value) as f64;
                let v = v as i16;
                d[0..2].copy_from_slice(&v.to_le_bytes());
                d[6] = 255;
                d[7] = 255;
                d
            }),
            _ => panic!("unsupported texture format"),
        };
        Image::new(
            Extent3d {
                width: self.size.x,
                height: self.size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            format,
        )
    }
    pub fn get_max_value(&self) -> Option<i16> {
        self.data
            .iter()
            .filter(|&&v| v != self.nodata_value)
            .max()
            .copied()
    }
    pub fn get_min_value(&self) -> Option<i16> {
        self.data
            .iter()
            .filter(|&&v| v != self.nodata_value)
            .min()
            .copied()
    }
    pub fn get(&self, pos: &UVec2) -> i16 {
        self.data[pos.y as usize * self.size.x as usize + pos.x as usize]
    }
    pub fn sample_nearest(&self, coord: &GeoCoord2) -> Option<i16> {
        let grid_index = self.space.to_grid_trunc(coord);
        if self.space.grid_size().bounds_check(grid_index.0) {
            let data = self.get(&grid_index.0);
            if data != self.nodata_value {
                Some(data)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct AsciiGridLoader;

#[derive(thiserror::Error, Debug)]
pub enum AsciiGridLoaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Parse(Error),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Header '{0}' missing")]
    HeaderMissing(&'static str),
    #[error("Header '{0}' duplicate")]
    HeaderDuplicate(&'static str),
    #[error("Header '{0}' unknown")]
    HeaderUnknown(String),
    #[error("Header '{0}' missing value")]
    HeaderMissingValue(&'static str),
    #[error("Header '{0}' invalid value: {1} ({2})")]
    HeaderInvalidValue(&'static str, String, String),
    #[error("Invalid data value: {0} ({1})")]
    InvalidDataValue(String, String),
    #[error("Unexpected end of column: expected {0}, found {1}")]
    UnexpectedEndOfCol(u32, u32),
    #[error("Unexpected end of file: expected {0} rows, found {1}")]
    UnexpectedEof(u32, u32),
    #[error("Expected end of file")]
    ExpectedEof,
}

impl AssetLoader for AsciiGridLoader {
    type Asset = AsciiGrid;
    type Error = AsciiGridLoaderError;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["asc"]
    }
    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _: &'a Self::Settings,
        _: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut data = String::new();
            reader.read_to_string(&mut data).await?;
            parse_ascii_asset(data.as_str()).map_err(AsciiGridLoaderError::Parse)
        })
    }
}

fn parse_header_line<T: FromStr>(
    line: &str,
    key: &'static str,
    value: &mut Option<T>,
) -> Result<bool, Error>
where
    T::Err: std::fmt::Display,
{
    let mut parts = line.split_ascii_whitespace();
    if parts.next() != Some(key) {
        return Ok(false);
    } else if value.is_some() {
        return Err(Error::HeaderDuplicate(key));
    }
    match parts.next() {
        Some(str) => {
            *value = Some(
                str.parse::<T>()
                    .map_err(|e| Error::HeaderInvalidValue(key, str.to_string(), e.to_string()))?,
            );
            Ok(true)
        }
        None => Err(Error::HeaderMissingValue(key)),
    }
}

fn parse_data_row(line: &str, cols: u32, data: &mut Vec<i16>) -> Result<(), Error> {
    let mut parts = line.split_ascii_whitespace();
    for col in 0u32..cols {
        let str = parts
            .next()
            .ok_or(Error::UnexpectedEndOfCol(cols, col.saturating_sub(1)))?;
        data.push(
            str.parse::<i16>()
                .map_err(|e| Error::InvalidDataValue(str.to_string(), e.to_string()))?,
        );
    }
    Ok(())
}

fn not_set<T>(name: &'static str, value: &Option<T>) -> Result<(), Error> {
    if value.is_none() {
        Err(Error::HeaderMissing(name))
    } else {
        Ok(())
    }
}

fn parse_ascii_asset(data: &str) -> Result<AsciiGrid, Error> {
    let mut lines = data.lines();

    let mut ncols = None;
    let mut nrows = None;
    let mut xllcorner = None;
    let mut yllcorner = None;
    let mut cellsize = None;
    let mut nodata_value = None;
    for _ in 0..6 {
        let Some(line) = lines.next() else {
            break;
        };
        if !(parse_header_line::<u32>(line, "ncols", &mut ncols)?
            || parse_header_line::<u32>(line, "nrows", &mut nrows)?
            || parse_header_line::<f64>(line, "xllcorner", &mut xllcorner)?
            || parse_header_line::<f64>(line, "yllcorner", &mut yllcorner)?
            || parse_header_line::<f64>(line, "cellsize", &mut cellsize)?
            || parse_header_line::<i16>(line, "NODATA_value", &mut nodata_value)?)
        {
            return Err(Error::HeaderUnknown(line.to_string()));
        }
    }
    not_set("ncols", &ncols)?;
    not_set("nrows", &nrows)?;
    not_set("xllcorner", &xllcorner)?;
    not_set("yllcorner", &yllcorner)?;
    not_set("cellsize", &cellsize)?;
    not_set("NODATA_value", &nodata_value)?;

    let grid_size = UVec2::new(ncols.unwrap(), nrows.unwrap());
    let cell_size = DVec2::splat(cellsize.unwrap());

    let mut data = Vec::with_capacity((grid_size.x * grid_size.y) as usize);
    for row in 0u32..grid_size.y {
        let line = lines
            .next()
            .ok_or(Error::UnexpectedEof(grid_size.x, row.saturating_sub(1)))?;
        parse_data_row(line, grid_size.x, &mut data)?;
    }
    if lines.next().is_some() {
        return Err(Error::ExpectedEof);
    }
    let ll_corner = DVec2::new(xllcorner.unwrap(), yllcorner.unwrap());
    let space = GeoGridSpace::from_ll_corner(ll_corner, grid_size.into(), cell_size.into());
    Ok(AsciiGrid {
        size: grid_size,
        ll_corner,
        cell_size: cell_size.x,
        nodata_value: nodata_value.unwrap(),
        space,
        data,
    })
}

#[derive(Default)]
pub struct AsciiGridPlugin;
impl Plugin for AsciiGridPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<AsciiGrid>()
            .register_asset_loader(AsciiGridLoader)
            .register_asset_reflect::<AsciiGrid>();
    }
}
