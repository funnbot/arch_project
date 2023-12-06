use super::map::Map;
use crate::{
    preludes::{core::*, math::*},
    GlobalRng,
};

struct Raindrop {
    coord: Vec2,
}

pub fn calculate_rain(map: &mut Map, rng: &mut GlobalRng) {
    let raindrops: Vec<Raindrop> = Vec::new();
    for cell in map.cell_land_iter_mut() {
        
    }
}
