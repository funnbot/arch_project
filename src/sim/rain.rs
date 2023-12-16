use std::{cell::Cell as StdCell, cmp::Ordering};

use bevy::utils::hashbrown::HashMap;
use bevy_egui::egui::epaint::util::OrderedFloat;
use rand::distributions::{Open01, Uniform};
use smallvec::SmallVec;

use super::{cell::Cell, config::Config, map::Map, n8};
use crate::{
    math::coord::MapCoord2,
    preludes::{bevy::*, core::*, math::*, rand::*},
    utils::cluster,
};

#[derive(Default, Clone, Debug, PartialEq)]
struct Raindrop {
    volume: StdCell<f32>,
}

// type RainMap = HashMap<UVec2, Raindrop>;

struct RainHashMap(HashMap<UVec2, f32>);
impl RainHashMap {
    fn add(&mut self, index2: UVec2, volume: f32) {
        *self.0.entry(index2).or_default() += volume;
    }
    fn get(&self, index2: UVec2) -> f32 {
        *self.0.get(&index2).unwrap_or(&0.0)
    }
}
struct RainMap {
    /// read buffer, for iterating and query
    read: RainHashMap,
    /// write buffer
    write: RainHashMap,
}
impl RainMap {
    fn with_capacity(count: usize) -> Self {
        Self {
            read: RainHashMap(HashMap::with_capacity(count)),
            write: RainHashMap(HashMap::with_capacity(count)),
        }
    }
    fn add(&mut self, index2: UVec2, volume: f32) {
        self.write.add(index2, volume);
    }
    fn get(&self, index2: UVec2) -> f32 {
        self.read.get(index2)
    }
    fn swap(&mut self) {
        std::mem::swap(&mut self.read, &mut self.write);
    }
    fn for_each<F: FnMut(&RainHashMap, &mut RainHashMap, UVec2, f32)>(&mut self, mut f: F) {
        let read: &RainHashMap = &self.read;
        let write: &mut RainHashMap = &mut self.write;
        for (&index2, &value) in read.0.iter() {
            f(read, write, index2, value);
        }
    }
}

pub fn calculate_rain<R: Rng>(map: &mut Map, config: &Config, rng: &mut R) {
    let rain_steps = config.rain_steps;
    let precip_percent = config.precip_percent;
    let infiltration = config.infiltration;
    let land_cell_count = 516000 as f32; // FIXME
    let initial_raindrop_count = (land_cell_count * precip_percent) as usize;
    let rainfall_mult = 1.0 / ((1_000_000.0 / precip_percent) * 10.0);

    for cell in map.grid.iter_land_mut() {
        cell.water_flow = 0.0;
    }

    let mut rain_map = RainMap::with_capacity(initial_raindrop_count);

    for cell in map.grid.rand_of_iter_land(rng, initial_raindrop_count) {
        let index2 = UVec2::from(cell.coord);
        rain_map.add(index2, cell.rainfall * rainfall_mult);
    }

    let n8_offsets = n8::zorder_n8_offsets();

    for _ in 0..rain_steps {
        rain_map.for_each(|read, write, index2, volume| {
            let (cell, n8s) = map.grid.get_cell_n8_mut(index2).unwrap();
            // TODO: instead of infiltration selecting a raindrop here at random (so random rain volumes are deleted), calculate some infiltration value since we have the current cell, based on soil or forest state? or based on current flow and rainfall of this cell.
            if rng.sample::<f32, _>(Open01) < infiltration {
                // the original implementation cant be replicated, this uses the current cells rainfall value, instead of the rainfall from the cell the raindrop was spawned on, in practice this shouldn't do much.
                write.add(index2, -cell.rainfall * rainfall_mult);
            }

            disperse(cell, &n8s, volume, read, write);
        });
        rain_map.swap();

        // rain_map.retain(|_, drop| drop.volume.get() > 0.0);

        // for drop in raindrops.iter_mut() {
        //     if rng.sample::<f32, _>(Open01) < infiltration {
        //         // discard this raindrop
        //         drop.destroyed = true;
        //         continue;
        //     }
        //     let (cell, n8s) = map.grid.get_cell_n8_mut(drop.index2).unwrap();
        // }
    }
    /*
    delete all raindrops; // unnecessary
    for all land cells, set flow 0;

    for precip_percent of land cells {
        create raindrop {
            volume = cell.rainfall / 1000000 / precip_percent * 10; // cubic km2 # conversion
        }
    }
    for rain_steps {
        raindrops.retain(|drop| {
            if rand(0,100) < config.infiltration {
                return false; // destroy raindrop
            }
            let (cell, n8s) = map.grid.get_cell_n8_mut(drop.coord);
            let target = n8s.iter().min_by(|n| {
                n.elevation +
            })
        });
    }
    */
}

/// get the current height, and all of the neighboring heights
/// disperse the current drop volume among the neighbors, leveling them out.
/// then the amount of volume remove from this cell will be added to the flow value.
fn disperse<'a, 'b>(
    cell: &'a mut Cell,
    n8s: &[Option<&'a mut Cell>; 8],
    volume: f32,
    read: &'b RainHashMap,
    write: &'b mut RainHashMap,
) {
    let mut height = cell.elevation as f32 + volume;
    let mut heights = [f32::MAX; 8];

    for (h, neigh) in heights.iter_mut().zip(n8s.iter()) {
        if let Some(neigh) = neigh {
            *h = neigh.elevation as f32 + read.get(neigh.coord.into());
        }
    }

    

    todo!()
}
