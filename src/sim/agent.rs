use bevy::transform::components::Transform;

use super::map::Map;
use crate::{
    math::coord::MapCoord2,
    preludes::{core::*, math::*, system::*},
};

#[derive(Component, Debug, Default)]
pub struct Agent {
    pub coord: MapCoord2,
}

pub fn apply_agent_transform_system(mut query: Query<(&Agent, &mut Transform)>) {
    for (agent, mut tform) in &mut query {
        tform.translation = agent.coord.0.as_vec2().extend(0.0)
    }
}
