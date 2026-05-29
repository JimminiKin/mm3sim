mod components;
mod resources;
mod systems;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};

use resources::programming_wheel_params::ProgrammingWheelParams;
use resources::carousel_params::{CarouselParams, CarouselState};
use resources::chute_params::{ChuteParams, MultiChuteConfig};
use resources::constants::BG_COLOR;
use resources::constants::SIMULATION_TPS;
use resources::hihat_params::{HiHatParams, HiHatState};
use resources::kick_params::KickParams;
use resources::marble_collisions::MarbleCollisions;
use resources::marble_runs::RunHistory;
use resources::ride_params::RideParams;
use resources::snare_params::SnareParams;
use resources::marble_params::MarbleParams;
use resources::power_params::PowerParams;
use resources::stats_intake::StatsIntake;
use resources::vibraphone_params::VibraphoneParams;
use systems::axes::{resize_axes_viewport, setup_axes_hud, update_axes_hud};
use systems::programming_wheel::{
    programming_wheel_editor_ui, programming_wheel_spawn_system, draw_programming_wheel_gizmos,
    rotate_programming_wheel_system, setup_programming_wheel_system, setup_spawners_system,
    sync_instrument_spawners,
};
use systems::camera::orbit_camera_system;
use systems::carousel::{animate_carousel_system, update_carousel_body, update_carousel_instruments};
use systems::chute_editor::{
    apply_snare_fixed_system, chute_editor_ui,
    rebuild_carousel_system, rebuild_chute_system, rebuild_hihat_system, rebuild_kick_system,
    rebuild_ride_system, rebuild_snare_system, rebuild_vibraphone_system,
    SnareFixed,
};
use systems::hud::hud_panel_ui;
use systems::power::power_panel_ui;
use systems::hihat::{sync_hihat_pedal_state, update_hihat_visual};
use systems::instrument::{detect_instrument_hits, record_instrument_hits, InstrumentHits};
use systems::marble::{
    advance_flight_timers_system, capture_prev_velocity_system,
    despawn_fallen_marbles_system, record_marble_paths_system,
    update_marble_collisions,
};
use systems::marble_graph::{
    draw_marble_ghosts_system, marble_graph_ui, record_marble_samples_system, snare_tip_graph_ui,
};
use systems::setup::setup_system;
use systems::sound::{play_instrument_sounds, setup_sounds, SnareVolume};

use components::carousel::spawn_carousel;
use components::hihat::spawn_hihat;
use components::kick::spawn_kick;
use components::ride::spawn_ride;

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
        .insert_resource(Time::<Fixed>::from_hz(SIMULATION_TPS.into()))
        .insert_resource(ClearColor(Color::srgb(BG_COLOR.0, BG_COLOR.1, BG_COLOR.2)))
        .init_resource::<ProgrammingWheelParams>()
        .init_resource::<ChuteParams>()
        .init_resource::<MultiChuteConfig>()
        .init_resource::<SnareParams>()
        .init_resource::<HiHatParams>()
        .init_resource::<KickParams>()
        .init_resource::<RideParams>()
        .init_resource::<CarouselParams>()
        .init_resource::<CarouselState>()
        .init_resource::<MarbleCollisions>()
        .init_resource::<MarbleParams>()
        .init_resource::<RunHistory>()
        .init_resource::<SnareFixed>()
        .init_resource::<SnareVolume>()
        .init_resource::<VibraphoneParams>()
        .init_resource::<InstrumentHits>()
        .init_resource::<HiHatState>()
        .init_resource::<PowerParams>()
        .init_resource::<StatsIntake>()
        .add_systems(
            Startup,
            (
                setup_system,
                setup_sounds,
                setup_axes_hud,
                |mut commands: Commands,
                 mut meshes: ResMut<Assets<Mesh>>,
                 mut materials: ResMut<Assets<StandardMaterial>>,
                 hihat_params: Res<HiHatParams>,
                 hihat_state: Res<HiHatState>,
                 kick_params: Res<KickParams>,
                 ride_params: Res<RideParams>,
                 carousel_params: Res<CarouselParams>| {
                    spawn_hihat(&mut commands, &mut meshes, &mut materials, &hihat_params, hihat_state.open);
                    spawn_kick(&mut commands, &mut meshes, &mut materials, &kick_params);
                    spawn_ride(&mut commands, &mut meshes, &mut materials, &ride_params);
                    spawn_carousel(&mut commands, &mut meshes, &mut materials, &carousel_params);
                },
            ),
        )
        .add_systems(PostStartup, (setup_programming_wheel_system, setup_spawners_system))
        .add_systems(
            FixedUpdate,
            capture_prev_velocity_system.before(PhysicsSystems::First),
        )
        .add_systems(
            FixedUpdate,
            (
                advance_flight_timers_system,
                detect_instrument_hits,
                record_instrument_hits,
                play_instrument_sounds,
                record_marble_samples_system
                    .run_if(|si: Res<StatsIntake>| si.0),
                record_marble_paths_system
                    .run_if(|si: Res<StatsIntake>| si.0),
            )
                .chain()
                .after(PhysicsSystems::Last),
        )
        .add_systems(
            Update,
            (
                orbit_camera_system,
                apply_snare_fixed_system,
                despawn_fallen_marbles_system,
                update_marble_collisions,
                draw_marble_ghosts_system,
                rebuild_snare_system,
                rebuild_chute_system,
                rebuild_vibraphone_system,
                rebuild_hihat_system,
                rebuild_kick_system,
                rebuild_ride_system,
                rebuild_carousel_system,
                animate_carousel_system,
                update_carousel_instruments,
                update_carousel_body,
                (
                    sync_instrument_spawners,
                    (
                        rotate_programming_wheel_system,
                        programming_wheel_spawn_system,
                        sync_hihat_pedal_state,
                        update_hihat_visual,
                    ).chain(),
                ).chain(),
                draw_programming_wheel_gizmos,
                update_axes_hud,
                resize_axes_viewport,
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                hud_panel_ui,
                chute_editor_ui,
                programming_wheel_editor_ui,
                marble_graph_ui,
                snare_tip_graph_ui,
                power_panel_ui,
            ),
        )
        .run();
}
