pub mod pipeline;
pub mod systems;

use bevy::prelude::Component;

/// Marks the entity as water.
#[derive(Default, Component)]
pub struct Water;
