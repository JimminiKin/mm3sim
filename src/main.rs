mod components;
mod resources;
mod systems;

use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};
use bevy_rapier3d::plugin::PhysicsSet;
use bevy_rapier3d::prelude::*;

use resources::chute_params::ChuteParams;
use resources::constants::BG_COLOR;
use resources::constants::SIMULATION_TPS;
use resources::marble_collisions::MarbleCollisions;
use resources::marble_runs::RunHistory;
use systems::axes::{resize_axes_viewport, setup_axes_hud, update_axes_hud};
use systems::camera::orbit_camera_system;
use systems::chute_editor::{
    apply_snare_fixed_system, chute_editor_ui, rebuild_chute_system, SnareFixed,
};
use systems::chute_handles::{
    chute_handle_drag_system, draw_chute_gizmos, setup_chute_handles, sync_handle_transforms,
    sync_handle_visibility, HandleDrag,
};
use systems::hud::hud_panel_ui;
use systems::marble::{
    advance_flight_timers_system, auto_spawn_system, capture_prev_velocity_system,
    despawn_fallen_marbles_system, record_marble_paths_system, record_snare_hit_system,
    spawn_marble_on_click_system, track_slide_end_system, update_marble_collisions, AutoSpawn,
};
use systems::marble_graph::{
    draw_marble_ghosts_system, marble_graph_ui, record_marble_samples_system,
};
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

    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule())
        .add_plugins(EguiPlugin { ..default() })
        .insert_resource(EguiGlobalSettings { auto_create_primary_context: false, ..default() })
        // 1000 Hz outer tick × 8 substeps = 8000 effective solver passes/s.
        // The substep dt (0.125 ms) moves a 4.4 m/s marble only 0.55 mm — below
        // Rapier's 1 mm allowed_linear_error, so Baumgarte position correction is
        // never triggered and bounces are driven purely by the restitution coefficient.
        .insert_resource(Time::<Fixed>::from_hz(SIMULATION_TPS.into()))
        .insert_resource(ClearColor(Color::srgb(BG_COLOR.0, BG_COLOR.1, BG_COLOR.2)))
        .init_resource::<ChuteParams>()
        .init_resource::<MarbleCollisions>()
        .init_resource::<HandleDrag>()
        .init_resource::<RunHistory>()
        .init_resource::<SnareFixed>()
        .init_resource::<SnareVolume>()
        .init_resource::<AutoSpawn>()
        .add_systems(
            Startup,
            (
                setup_system,
                setup_snare_sound,
                setup_axes_hud,
                setup_chute_handles,
            ),
        )
        // Snapshot velocity before physics so impact recording always sees approach speed,
        // not the partial post-bounce velocity that varies by substep contact phase.
        .add_systems(
            FixedUpdate,
            capture_prev_velocity_system.before(PhysicsSet::SyncBackend),
        )
        // All physics data collection runs after PhysicsSet::Writeback so collision events
        // and velocity reads come from the completed step, not a prior tick's state.
        .add_systems(
            FixedUpdate,
            (
                advance_flight_timers_system,
                track_slide_end_system,
                record_snare_hit_system,
                record_marble_samples_system,
                record_marble_paths_system,
            )
                .chain()
                .after(PhysicsSet::Writeback),
        )
        .add_systems(
            Update,
            (
                // Input
                chute_handle_drag_system,
                orbit_camera_system,
                spawn_marble_on_click_system.after(chute_handle_drag_system),
                // Simulation maintenance
                apply_snare_fixed_system,
                despawn_fallen_marbles_system,
                update_marble_collisions,
                snare_hit_sound_system,
                draw_marble_ghosts_system,
                // Chute
                rebuild_chute_system,
                sync_handle_transforms,
                sync_handle_visibility,
                draw_chute_gizmos,
                // Axes overlay
                update_axes_hud,
                resize_axes_viewport,
            ),
        )
        // UI systems run inside EguiPrimaryContextPass (PostUpdate), which is when
        // bevy_egui's multipass mode initialises the context for the current frame.
        .add_systems(
            EguiPrimaryContextPass,
            (
                hud_panel_ui,
                chute_editor_ui,
                marble_graph_ui,
                // auto_spawn reads Stop/Start state written by chute_editor_ui this same pass
                auto_spawn_system.after(chute_editor_ui),
            ),
        )
        .run();
}
