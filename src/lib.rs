#![feature(never_type)]

pub mod ascii_grid;
pub mod math;
pub mod pid_control;
pub mod sim;
pub mod ui;
pub mod utils;

mod rand;
mod schedule;

pub use rand::{GlobalRng, RngComponent};
pub use schedule::{MyAssets, MyPlugin, MyStates, MyUpdate};

pub(crate) mod preludes {
    pub mod core {
        pub use bevy::log::{debug, error, info, warn};
        pub use bevy::reflect;
        pub use bevy::utils::default;
        pub use thiserror::Error;
    }
    pub mod app {
        pub use bevy::app::App;
        pub use bevy::DefaultPlugins;
    }
    pub mod plugin {
        pub use bevy::app::{App, Plugin};
    }
    pub mod asset {
        pub use bevy::asset::{AssetEvent, AssetServer, Assets, Handle};

        pub use crate::ascii_grid::AsciiGrid;
    }
    pub mod system {
        pub use bevy::app::{Startup, Update};
        pub use bevy::ecs::bundle;
        pub use bevy::ecs::component::Component;
        pub use bevy::ecs::entity::Entity;
        pub use bevy::ecs::event::EventReader;
        pub use bevy::ecs::query::{With, Without};
        pub use bevy::ecs::reflect::{ReflectComponent, ReflectResource};
        pub use bevy::ecs::system::{Commands, Query, Res, ResMut, Resource};
        pub use bevy::ecs::world::World;
    }
    pub mod cond {
        pub use bevy::ecs::schedule::common_conditions::*;
        pub use bevy::ecs::schedule::{Condition, IntoSystemConfigs, IntoSystemSetConfigs};
    }
    pub mod math {
        pub use bevy::math::{DAffine2, DVec2, UVec2, Vec2, Vec3};

        pub use crate::math::coord::GridTransform as _;
        pub use crate::math::DRect;
    }
    pub mod image {
        pub use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
        pub use bevy::render::texture::Image;
        pub use image::DynamicImage;
    }
    pub mod render {
        pub use bevy::core_pipeline::core_2d::Camera2dBundle;
    }
}
