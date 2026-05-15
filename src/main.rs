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
use systems::sound::{setup_snare_sound, snare_hit_sound_system};
use systems::hud::{setup_hud, update_hud_system, record_snare_aoa_system};

fn main() {
    let mut app = App::new();

    // AudioPlugin uses rodio's WAV decoder which panics on WASM — disable it
    // there and drive audio through the web_sys AudioContext path instead.
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::audio::AudioPlugin>(),
    );
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DefaultPlugins);

    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(ClearColor(Color::rgb(BG_COLOR.0, BG_COLOR.1, BG_COLOR.2)))
        .add_systems(Startup, (setup_system, setup_snare_sound, setup_hud))
        .add_systems(
            Update,
            (
                spawn_marble_on_click_system,
                orbit_camera_system,
                despawn_fallen_marbles_system,
                draw_axes_system,
                snare_hit_sound_system,
                record_snare_aoa_system,
                update_hud_system,
            ),
        )
        .run();
}
