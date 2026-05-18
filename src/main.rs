mod components;
mod resources;
mod systems;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_rapier3d::plugin::PhysicsSet;
use bevy_rapier3d::prelude::*;

use resources::chute_params::ChuteParams;
use resources::constants::BG_COLOR;
use resources::marble_collisions::MarbleCollisions;
use resources::marble_runs::RunHistory;
use systems::axes::{resize_axes_viewport, setup_axes_hud, update_axes_hud};
use systems::camera::orbit_camera_system;
use systems::chute_editor::{apply_snare_fixed_system, chute_editor_ui, rebuild_chute_system, SnareFixed};
use systems::chute_handles::{
    chute_handle_drag_system, draw_chute_gizmos, setup_chute_handles, sync_handle_transforms,
    sync_handle_visibility, HandleDrag,
};
use systems::hud::hud_panel_ui;
use systems::marble::{
    despawn_fallen_marbles_system, record_marble_paths_system, record_snare_hit_system,
    spawn_marble_on_click_system, track_slide_end_system, update_marble_collisions,
};
use systems::marble_graph::{draw_marble_ghosts_system, marble_graph_ui, record_chute_marble_system};
use systems::setup::setup_system;
use systems::sound::{setup_snare_sound, snare_hit_sound_system, SnareVolume};

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
        // 2000 Hz physics: reduces step-boundary collision time quantization to ~0.50 ms max.
        .insert_resource(Time::<Fixed>::from_hz(2000.0))
        .insert_resource(ClearColor(Color::srgb(BG_COLOR.0, BG_COLOR.1, BG_COLOR.2)))
        .init_resource::<ChuteParams>()
        .init_resource::<MarbleCollisions>()
        .init_resource::<HandleDrag>()
        .init_resource::<RunHistory>()
        .init_resource::<SnareFixed>()
        .init_resource::<SnareVolume>()
        .add_systems(
            Startup,
            (
                setup_system,
                setup_snare_sound,
                setup_axes_hud,
                setup_chute_handles,
            ),
        )
        // All physics data collection runs after PhysicsSet::Writeback so collision events
        // and velocity reads come from the completed step, not a prior tick's state.
        .add_systems(
            FixedUpdate,
            (
                track_slide_end_system,
                record_snare_hit_system,
                record_chute_marble_system,
            )
                .after(PhysicsSet::Writeback),
        )
        .add_systems(
            Update,
            (
                // Input
                chute_handle_drag_system,
                spawn_marble_on_click_system.after(chute_handle_drag_system),
                orbit_camera_system,
                // Simulation maintenance
                apply_snare_fixed_system,
                despawn_fallen_marbles_system,
                update_marble_collisions,
                snare_hit_sound_system,
                record_marble_paths_system,
                draw_marble_ghosts_system,
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
