#![feature(never_type)]
#![feature(effects)]
#![feature(slice_partition_dedup)]

pub mod ascii_grid;
pub mod math;
pub mod pid_control;
pub mod sim;
pub mod ui;
pub mod utils;

mod schedule;

pub use schedule::{MyAssets, MyPlugin, MyStates, MyUpdate};

pub(crate) mod preludes {
    pub mod core {
        pub use bevy::{
            log::{debug, error, info, warn},
            reflect,
            utils::default,
        };
        pub use thiserror::Error;
    }
    pub mod bevy {
        pub use bevy::{
            app::{App, Plugin, Startup, Update},
            asset::{AssetEvent, AssetServer, Assets, Handle},
            ecs::{
                bundle,
                component::Component,
                entity::Entity,
                event::EventReader,
                query::{With, Without},
                reflect::{ReflectComponent, ReflectResource},
                system::{Commands, Query, Res, ResMut, Resource},
                world::World,
            },
            math::{DVec2, UVec2, Vec2, Vec3},
            render::{
                render_resource::{Extent3d, TextureDimension, TextureFormat},
                texture::Image,
            },
        };
        use bevy_rand::{component::EntropyComponent, prelude::WyRand, resource::GlobalEntropy};
        pub use rand::Rng;

        pub type GlobalRng = GlobalEntropy<WyRand>;
        pub type RngComponent = EntropyComponent<WyRand>;
    }
    pub mod app {
        pub use bevy::{app::App, DefaultPlugins};
    }
    pub mod plugin {
        pub use bevy::app::{App, Plugin};
    }
    pub mod asset {
        pub use bevy::asset::{AssetEvent, AssetServer, Assets, Handle};

        pub use crate::ascii_grid::AsciiGrid;
    }
    pub mod system {
        pub use bevy::{
            app::{Startup, Update},
            ecs::{
                bundle,
                component::Component,
                entity::Entity,
                event::EventReader,
                query::{With, Without},
                reflect::{ReflectComponent, ReflectResource},
                system::{Commands, Query, Res, ResMut, Resource},
                world::World,
            },
        };
    }
    pub mod cond {
        pub use bevy::ecs::schedule::{
            common_conditions::*, Condition, IntoSystemConfigs, IntoSystemSetConfigs,
        };
    }
    pub mod math {
        pub use bevy::math::{DAffine2, DVec2, UVec2, Vec2, Vec3};

        pub use crate::math::{coord::GridTransform as _, DRect};
    }
    pub mod image {
        pub use bevy::render::{
            render_resource::{Extent3d, TextureDimension, TextureFormat},
            texture::Image,
        };
        pub use image::DynamicImage;
    }
    pub mod render {
        pub use bevy::core_pipeline::core_2d::Camera2dBundle;
    }
    pub mod rand {
        use bevy_rand::{component::EntropyComponent, prelude::WyRand, resource::GlobalEntropy};
        pub use rand::Rng;

        pub type GlobalRng = GlobalEntropy<WyRand>;
        pub type RngComponent = EntropyComponent<WyRand>;
    }
}
