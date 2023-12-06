use super::{config::Config, map::Map, SimulatedTime};
use crate::preludes::{core::*, system::*};

#[derive(reflect::Reflect, Default, Debug, Resource)]
#[reflect(Resource)]
pub struct ClimateCycle {
    counter: i32,
    rainfall_change: f32,
}

pub fn update_climate_scenario(
    mut map: ResMut<Map>,
    mut cycle: ResMut<ClimateCycle>,
    time: Res<SimulatedTime>,
    config: Res<Config>,
) {
    if config.enable_climate_cycle {
        cycle.counter += 1;
        let cvar = config.climate_var;
        if cycle.counter == cvar * 8
            || cycle.counter == cvar * 3
            || cycle.counter == cvar * 2
            || cycle.counter == cvar * 1
        {
            cycle.rainfall_change -= config.rain_change;
        } else if cycle.counter == cvar * 7
            || cycle.counter == cvar * 6
            || cycle.counter == cvar * 5
            || cycle.counter == cvar * 4
        {
            cycle.rainfall_change += config.rain_change;
        }
        if cycle.counter >= cvar * 8 {
            cycle.counter = 0;
        }
    }
    if config.enable_climate_change {
        for (cell, n8s) in map.cell_n8_iter_mut(|c| c.is_land) {
            let rain_mult = 1.0 + cycle.rainfall_change;
            let cleared_rainfall_effects =
                n8s.iter().flatten().filter(|c| c.forest_state == 1).count() as f32
                    * config.veg_rainfall;
            cell.rainfall = (cell.original_rainfall * rain_mult) - cleared_rainfall_effects;
        }
    }
}
