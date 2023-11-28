mod ascii_asset;
mod drect;
mod mayasim;
mod pid_control;
mod sim;
mod sim_time;
mod update_set;
mod zcoord;
mod coord;

use std::time::Duration;

use ascii_asset::AsciiAssetPlugin;
use bevy::asset::AssetServer;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::{AutomaticUpdate, SpatialAccess, TransformMode};

use crate::pid_control::Pid32;
use crate::update_set::UpdateSet;

#[derive(Component, Default)]
struct TrackedByKDTree;

type NNTree = KDTree2<TrackedByKDTree>;

const EPS: f32 = 0.00001;
const BOID_DIAG_LENGTH: f32 = 3.53553390593; //sqrt(2)*2.5
const BOID_DIAG_LEN_RECIP: f32 = 0.28284271247; //1/(sqrt(2)*2.5)

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..default()
            }),
        }))
        .add_plugins(
            AutomaticUpdate::<TrackedByKDTree>::new()
                .with_frequency(Duration::from_secs_f32(0.3))
                .with_transform(TransformMode::GlobalTransform)
                .with_spatial_ds(bevy_spatial::SpatialStructure::KDTree2),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(AsciiAssetPlugin)
        .configure_sets(
            Update,
            (
                UpdateSet::Pre,
                UpdateSet::Sim,
                UpdateSet::Post,
                UpdateSet::Egui,
            ),
        )
        .add_plugins(sim_time::SimTimePlugin)
        .add_plugins(sim::SimPlugin)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    // let tex = assets.load("arrow.png");
    // for i in 0..200 {
    //     commands.spawn(HumanBundle {
    //         agent: HumanAgent,
    //         gfx: SpriteBundle {
    //             transform: Transform::from_xyz((i % 10) as f32 * 15.0, (i / 10) as f32 * 15.0, 0.0),
    //             texture: tex.clone(),
    //             ..default()
    //         },
    //         tracked: TrackedByKDTree,
    //     });
    // }
}

#[derive(Bundle)]
struct HumanBundle {
    agent: HumanAgent,
    gfx: SpriteBundle,
    tracked: TrackedByKDTree,
}

#[derive(Component, Debug, Default)]
struct Kinematics2d {
    velocity: Vec2,
    angular_velocity: f32,
}

#[derive(Component, Debug)]
struct HumanAgent;

fn avoid_neighbors(mut q: Query<&mut Transform, With<HumanAgent>>, nn: Res<NNTree>) {
    q.par_iter_mut().for_each(|mut tform| {
        let mut force = Vec2::ZERO;
        for (n_pos, _) in nn.within_distance(tform.translation.xy(), 4.0) {
            let diff = tform.translation.xy() - n_pos;
            let dir_away = diff.normalize_or_zero();
            force -= dir_away
                / (1. + BOID_DIAG_LEN_RECIP * (diff.length_squared() - BOID_DIAG_LENGTH).exp());
        }
        if force.length_squared() > EPS {
            tform.translation += force.normalize_or_zero().extend(0.0);
        }
    })
}

fn human_agent_update(
    mut q: Query<(&mut HumanAgent, &mut Transform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let mouse_pos = match q_windows.single().cursor_position() {
        Some(v) => v,
        None => return,
    };

    let (cam, cam_tform) = q_camera.single();

    let mouse_world = match cam.viewport_to_world_2d(cam_tform, mouse_pos) {
        Some(v) => v.extend(0.0),
        None => return,
    };

    q.par_iter_mut().for_each(|(mut agent, mut tform)| {
        //*tform = tform.with_rotation(Quat::from_rotation_z(1.0) * tform.rotation);
        tform.translation = tform.translation.lerp(mouse_world, 0.01);
    })
}
