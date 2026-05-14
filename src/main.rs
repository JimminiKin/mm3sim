mod components;
mod resources;
mod systems;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use resources::constants::BG_COLOR;
use systems::setup::setup_system;
use systems::marble::{spawn_marble_on_click_system, despawn_fallen_marbles_system};
use systems::camera::orbit_camera_system;
use systems::axes::draw_axes_system;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(ClearColor(Color::rgb(BG_COLOR.0, BG_COLOR.1, BG_COLOR.2)))
        .add_systems(Startup, setup_system)
        .add_systems(Update, (spawn_marble_on_click_system, orbit_camera_system, despawn_fallen_marbles_system, draw_axes_system))
        .run();
}
