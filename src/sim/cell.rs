use bevy::render::color::Color;

use crate::{
    math::{coord::MapIndex2, ZCoord},
    preludes::{core::*, system::*},
};

/// Mostly static precomputed data about a grid location.
/// aka a patch.
#[derive(Debug, reflect::Reflect, Default, Clone)]
pub struct Cell {
    pub coord: ZCoord,
    pub color: Color,
    pub child: CellChild,

    pub elevation: i32,
    pub temperature: f32,
    pub original_rainfall: f32,
    pub rainfall: f32,
    pub soil_prod: f32,

    pub slope: f32,
    pub water_flow: f32,

    pub is_land: bool,
    pub is_agri: bool,
    pub is_vacant: bool,
    /// is border with water
    /// sum [is-land-patch] of neighbors < 6
    pub is_border: bool,

    pub agri_impact: f32,
    pub agri_suitability: f32,
    pub crop_yield: i32,
    /// benefit cost ratio of agriculture
    pub bca_agri: f32,
    pub cropping_value: f32,

    pub env_degrade: f32,
    pub npp: f64, // net primary productivity
    pub rain_value: f32,

    pub forest_state: i32,
    pub succession_counter: i32,
    pub forest_food_value: f32,

    pub pop_gradient: f32,

    pub travel_cost: f32,
    pub overland_to: f32,
    pub freshwater_to: f32,

    pub eco_services_value: f32,

    pub cell_migrant_util: (),
    pub travel_cost_util: (),
    pub eco_services_util: (),
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
