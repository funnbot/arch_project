use bevy::render::color::Color;

use crate::math::coord::MapIndex2;
use crate::math::ZCoord;
use crate::preludes::core::*;
use crate::preludes::system::*;

/// Mostly static precomputed data about a grid location.
/// aka a patch.
#[derive(Debug, reflect::Reflect, Default, Clone)]
pub struct Cell {
    pub coord: ZCoord,
    pub color: Color,
    pub child: CellChild,

    pub elevation: i32,
    pub temperature: f32,
    pub original_rainfall: i32,
    pub rainfall: i32,
    pub soil_prod: f32,

    pub slope: f64,

    pub is_land: bool,
}

#[derive(Debug, reflect::Reflect, Default, Clone)]
pub enum CellChild {
    Settlement(Entity),
    Agriculture(Entity),
    #[default]
    Empty,
}

impl Cell {
    pub fn coord(&self) -> MapIndex2 {
        MapIndex2(self.coord.into())
    }
}
