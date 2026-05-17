mod components;
mod resources;
mod systems;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_rapier3d::prelude::*;

use resources::constants::BG_COLOR;
use resources::chute_params::ChuteParams;
use resources::marble_collisions::MarbleCollisions;
use resources::marble_runs::RunHistory;
use systems::setup::setup_system;
use systems::marble::{
    despawn_fallen_marbles_system, record_snare_hit_system, setup_marble_trail_assets,
    spawn_marble_on_click_system, track_slide_end_system, trail_record_system,
    update_marble_collisions,
};
use systems::camera::orbit_camera_system;
use systems::axes::{resize_axes_viewport, setup_axes_hud, update_axes_hud};
use systems::sound::{setup_snare_sound, snare_hit_sound_system};
use systems::hud::hud_panel_ui;
use systems::chute_editor::{chute_editor_ui, rebuild_chute_system};
use systems::marble_graph::{marble_graph_ui, record_chute_marble_system};
use systems::chute_handles::{
    HandleDrag, chute_handle_drag_system, draw_chute_gizmos, setup_chute_handles,
    sync_handle_transforms, sync_handle_visibility,
};

fn main() {
    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::audio::AudioPlugin>()
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
    );
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DefaultPlugins);

    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(EguiPlugin)
        .insert_resource(ClearColor(Color::srgb(BG_COLOR.0, BG_COLOR.1, BG_COLOR.2)))
        .init_resource::<ChuteParams>()
        .init_resource::<MarbleCollisions>()
        .init_resource::<HandleDrag>()
        .init_resource::<RunHistory>()
        .add_systems(
            Startup,
            (setup_system, setup_snare_sound, setup_axes_hud, setup_chute_handles, setup_marble_trail_assets),
        )
        .add_systems(
            Update,
            (
                // Input
                chute_handle_drag_system,
                spawn_marble_on_click_system.after(chute_handle_drag_system),
                orbit_camera_system,
                // Physics data collection
                track_slide_end_system,
                record_snare_hit_system,
                record_chute_marble_system,
                // Simulation maintenance
                despawn_fallen_marbles_system,
                update_marble_collisions,
                snare_hit_sound_system,
                trail_record_system,
                // Chute
                rebuild_chute_system,
                sync_handle_transforms,
                sync_handle_visibility,
                draw_chute_gizmos,
                // UI
                hud_panel_ui,
                chute_editor_ui,
                marble_graph_ui,
                // Axes overlay
                update_axes_hud,
                resize_axes_viewport,
            ),
        )
        .run();
}
