mod components;
mod resources;
mod systems;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use systems::setup::setup_system;
use systems::marble::spawn_marble_on_click_system;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.08)))
        .add_systems(Startup, setup_system)
        .add_systems(Update, spawn_marble_on_click_system)
        .run();
}
