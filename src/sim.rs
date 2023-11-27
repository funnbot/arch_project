use bevy::prelude::*;

use crate::ascii_asset::AsciiAsset;
use crate::drect::{CoordinateTransform, DRect, GridDimensions};

#[derive(Resource)]
pub struct Map {
    cells: Vec<Cell>,
    dim: GridDimensions,
    transformer: CoordinateTransform,
}

impl Map {
    pub fn set_transformer(&mut self, rect: DRect) {
        self.transformer = CoordinateTransform::new(rect, *self.dim.rect(), true);
    }
}

pub enum CellChild {
    Settlement(Entity),
    Agriculture(Entity),
    Empty,
}

/// Mostly static precomputed data about a grid location
/// aka a patch
struct Cell {
    elevation: f32,
    child: CellChild,
}
