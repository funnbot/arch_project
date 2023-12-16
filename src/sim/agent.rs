use std::{marker::PhantomData, time::Duration};

use bevy::{
    app::Plugins, ecs::change_detection::Ref, reflect::TypePath, sprite::SpriteBundle,
    transform::components::Transform,
};
use bevy_spatial::{kdtree::KDTree2, AutomaticUpdate, SpatialStructure};
use kd_tree::KdTreeN;

use super::{map::Map, SimUpdate};
use crate::{
    math::coord::MapCoord2,
    preludes::{bevy::*, core::*, math::*, system::*},
};

#[derive(reflect::Reflect, Component, Debug, Default, Clone)]
#[reflect(Component)]
pub struct Agent {
    pub coord: DVec2,
}

#[derive(bundle::Bundle, Clone, Default)]
pub struct AgentBundle<T: Component + Default> {
    pub gfx: SpriteBundle,
    pub agent: Agent,
    pub comp: T,
}

pub fn apply_agent_transform_system(mut query: Query<(&Agent, &mut Transform)>) {
    for (agent, mut tform) in &mut query {
        tform.translation = agent.coord.as_vec2().extend(0.0)
    }
}

pub fn register_agent<T: Component + reflect::GetTypeRegistration + Default + TypePath>(
    app: &mut App,
) {
    app.register_type::<T>();
    app.init_resource::<KDTree2<T>>();
    app.add_systems(SimUpdate, update_spatial_system::<T>);
}

pub fn update_spatial_system<T: Component>(
    mut tree: ResMut<KDTree2<T>>,
    query: Query<(Entity, Ref<Transform>), With<T>>,
) {
    tree.tree = KdTreeN::par_build_by_ordered_float(
        query
            .iter()
            .map(|(e, tf)| (e, tf.translation.truncate()).into())
            .collect(),
    );
}
