mod agent;
mod cell;
mod climate;
mod config;
mod map;
mod n8;
mod rain;
mod time;

use bevy::app::{App, Plugin};
pub use map::MapPlugin;
pub use time::{SimTimePlugin, SimUpdate, SimulatedTime};

pub struct SimPlugin;
impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SimTimePlugin)
            .add_plugins(MapPlugin)
            .register_type::<config::Config>();
    }
}
