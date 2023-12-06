use bevy::{
    audio::Decodable,
    ecs::{reflect::ReflectResource, system::Resource},
};
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay, prelude::ReflectInspectorOptions,
    InspectorOptions,
};

use crate::preludes::core::*;

#[derive(reflect::Reflect, Resource, Debug, Clone, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct Config {
    pub enable_nature: bool,
    pub enable_rain: bool,
    pub enable_humans: bool,
    pub enable_climate_cycle: bool,
    pub enable_climate_change: bool,
    pub enable_influence_view: bool,
    pub enable_agri_view: bool,

    // anrthopogenic
    pub enable_migration: bool,
    pub enable_trade: bool,
    //   settlements
    #[inspector(min = 1, max = 20)]
    pub initial_settlement_count: i32,
    pub rank_1_pop: i32,
    pub rank_2_pop: i32,
    pub rank_3_pop: i32,
    //   migration
    pub travel_cost_pref: f32,
    pub eco_service_pref: f32,
    //   travel cost
    pub water_flow_cost: f32,
    pub slope_cost: f32,
    //   real income
    pub trade_value: i32,
    pub eco_service_value: i32,
    pub agri_value: f32,
    pub population_control: bool,
    // biophysical
    //   climate
    pub rain_change: f32,
    pub veg_rainfall: f32,
    #[inspector(min = 0, max = 15, display = NumberDisplay::Slider)]
    pub climate_var: i32,
    //   water
    pub rain_steps: i32,
    pub precip_percent: f32,
    pub infiltration: f32,
    //   soils
    pub soil_deg_rate: f32,
    pub soil_regen_rate: f32,
    //   agriculture
    pub agri_estab_cost: i32,
    pub agri_travel_cost: i32,
    pub agri_suit_npp: i32,
    pub agri_suit_slope: i32,
    pub agri_suit_soils: i32,
    pub agri_suit_flow: i32,
    //   crop yield
    pub crop_max_yield: i32,
    pub crop_origin_shift: f32,
    pub copy_slope_yield: f32,
    //   forests
    pub forest_disturb_rate: f32,
    pub forest_state_change_s2: i32,
    pub forest_state_change_s3: i32,
    pub forst_s3num_neigh: i32,
    //   ecosystem services
    pub crop_value_param: f32,
    pub forest_value_param: f32,
    pub flow_value_param: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self { ..default() }
    }
}
