use bevy::prelude::SystemSet;

#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum UpdateSet {
    Pre,
    Sim,
    Post,
    Egui,
}
