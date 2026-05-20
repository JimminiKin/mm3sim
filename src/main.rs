mod components;
mod resources;
mod systems;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};

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
    draw_marble_ghosts_system, marble_graph_ui, record_marble_samples_system, snare_tip_graph_ui,
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

    app.add_plugins(PhysicsPlugins::default())
        .add_plugins(EguiPlugin { ..default() })
        .insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        })
        // 2000 Hz fixed timestep → 0.5 ms timing resolution for sub-millisecond measurements.
        // Avian's XPBD solver is unconditionally stable at any timestep, so no CCD
        // threshold or Baumgarte tuning is needed.
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
        // Snapshot velocity before physics so impact recording always sees approach speed.
        .add_systems(
            FixedUpdate,
            capture_prev_velocity_system.before(PhysicsSystems::First),
        )
        // All physics data collection runs after physics writeback.
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
                .after(PhysicsSystems::Last),
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
        .add_systems(
            EguiPrimaryContextPass,
            (
                hud_panel_ui,
                chute_editor_ui,
                marble_graph_ui,
                snare_tip_graph_ui,
                auto_spawn_system.after(chute_editor_ui),
            ),
        )
        .run();
}
