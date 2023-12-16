use std::time::Duration;

use arch_project::{sim::SimUpdate, *};
use ascii_grid::AsciiGridPlugin;
use bevy::{
    asset::AssetServer,
    ecs::{
        query::{ReadOnlyWorldQuery, WorldQuery},
        schedule::ScheduleLabel,
    },
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand};
use bevy_spatial::{kdtree::KDTree2, AutomaticUpdate, SpatialAccess, TransformMode};

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
        .add_plugins(EntropyPlugin::<WyRand>::with_seed([13u8; 8]))
        .add_plugins(EguiPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(AsciiGridPlugin)
        .add_plugins(MyPlugin)
        .add_plugins(sim::SimPlugin)
        .add_systems(Startup, (startup, crate::ui::set_egui_style_system))
        .add_systems(Update, crate::ui::camera::camera_system)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
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
