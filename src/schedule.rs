use bevy::asset::{Asset, AssetPath, UntypedAssetId};
use bevy::ecs::schedule::{IntoSystemSetConfigs, NextState, OnEnter};
use bevy::ecs::world::FromWorld;
use bevy::prelude::{States, SystemSet};

use crate::preludes::asset::*;
use crate::preludes::cond::*;
use crate::preludes::core::*;
use crate::preludes::image::*;
use crate::preludes::plugin::*;
use crate::preludes::system::*;

#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum MyUpdate {
    /// run before the simulation
    Pre,
    /// set which runs the SimUpdate schedule
    Sim,
    /// run after the simulation
    Post,
    /// run after post
    UI,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum MyStates {
    #[default]
    Loading,
    Running,
}

#[derive(Resource, reflect::Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct MyAssets {
    pub temperature: Handle<AsciiGrid>,
    pub elevation: Handle<AsciiGrid>,
    pub precipitation: Handle<AsciiGrid>,
    pub soil_productivity: Handle<AsciiGrid>,
    pub background: Handle<Image>,
    pub arrow: Handle<Image>,
}

#[derive(Resource, Debug)]
pub struct AssetLoadingTracker {
    loading: Vec<UntypedAssetId>,
    server: AssetServer,
}
impl FromWorld for AssetLoadingTracker {
    fn from_world(world: &mut World) -> Self {
        Self {
            loading: Vec::new(),
            server: world.resource::<AssetServer>().clone(),
        }
    }
}

impl AssetLoadingTracker {
    fn load<'a, A: Asset>(&mut self, path: impl Into<AssetPath<'a>>) -> Handle<A> {
        let handle = self.server.load(path);
        self.loading.push(handle.id().into());
        handle
    }
    fn is_all_loaded(&mut self) -> bool {
        self.loading
            .retain(|id| !self.server.is_loaded_with_dependencies(*id));

        self.loading.is_empty()
    }
}

fn load_assets_system(mut my_assets: ResMut<MyAssets>, mut server: ResMut<AssetLoadingTracker>) {
    my_assets.temperature = server.load("temp.asc");
    my_assets.elevation = server.load("elevation.asc");
    my_assets.precipitation = server.load("precip.asc");
    my_assets.soil_productivity = server.load("soil.asc");
    my_assets.background = server.load("Maya.png");
    my_assets.arrow = server.load("arrow.png");
}

fn loading_progress_system(
    mut tracker: ResMut<AssetLoadingTracker>,
    mut next_state: ResMut<NextState<MyStates>>,
) {
    if tracker.is_all_loaded() {
        next_state.set(MyStates::Running);
    }
}

pub struct MyPlugin;
impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MyStates>()
            .configure_sets(
                Update,
                (MyUpdate::Pre, MyUpdate::Sim, MyUpdate::Post, MyUpdate::UI)
                    .chain()
                    .run_if(in_state(MyStates::Running)),
            )
            .init_resource::<MyAssets>()
            .init_resource::<AssetLoadingTracker>()
            .add_systems(Startup, load_assets_system)
            .add_systems(
                Update,
                loading_progress_system.run_if(in_state(MyStates::Loading)),
            );
    }
}
