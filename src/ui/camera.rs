use bevy::core_pipeline::core_2d::{Camera2d, Camera2dBundle};
use bevy::ecs::event::EventReader;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::{MouseButton, MouseMotion, MouseWheel};
use bevy::input::Input;
use bevy::render::camera::{Camera, OrthographicProjection, RenderTarget};
use bevy::transform::components::Transform;
use bevy::window::WindowRef;

use crate::math::coord::{ScreenIndex2, ScreenSpace};
use crate::preludes::core::*;
use crate::preludes::math::*;
use crate::preludes::system::*;

#[derive(Component, Debug, Clone)]
pub struct CameraControl {
    pub space: ScreenSpace,
    pub zoom: f32,
}

pub fn camera_system(
    mut camera: Query<(
        &Camera,
        &Camera2d,
        &mut OrthographicProjection,
        &mut Transform,
    )>,
    mut on_mouse_wheel: EventReader<MouseWheel>,
    mut on_mouse_motion: EventReader<MouseMotion>,
    input: ResMut<Input<MouseButton>>,
) {
    let mut translate = Vec2::ZERO;
    for motion in on_mouse_motion.read() {
        if input.pressed(MouseButton::Middle) {
            translate += motion.delta * Vec2::new(-1.0, 1.0);
        }
    }

    let mut vertical_scroll: f32 = 0.0;
    for wheel_event in on_mouse_wheel.read() {
        vertical_scroll += wheel_event.y;
    }

    for (cam, cam2d, mut ortho, mut tform) in &mut camera {
        if let RenderTarget::Window(WindowRef::Primary) = cam.target {
            ortho.scale -= vertical_scroll * 0.02;
            ortho.scale = ortho.scale.max(0.01);
            tform.translation += translate.extend(0.0) * ortho.scale;
        }
    }
}
