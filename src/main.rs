mod components;
mod resources;
mod systems;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_rapier3d::prelude::*;

use resources::constants::BG_COLOR;
use resources::chute_params::ChuteParams;
use resources::marble_collisions::MarbleCollisions;
use resources::marble_runs::AllMarbleRuns;
use systems::setup::setup_system;
use systems::marble::{
    spawn_marble_on_click_system, despawn_fallen_marbles_system,
    update_marble_collisions, setup_marble_trail_assets, trail_record_system,
};
use systems::camera::orbit_camera_system;
use systems::axes::{setup_axes_hud, update_axes_hud, resize_axes_viewport};
use systems::sound::{setup_snare_sound, snare_hit_sound_system};
use systems::hud::{hud_panel_ui, record_snare_aoa_system, track_slide_end_system};
use systems::chute_editor::{chute_editor_ui, rebuild_chute_system};
use systems::marble_graph::{record_chute_marble_system, marble_graph_ui};
use systems::chute_handles::{
    HandleDrag, setup_chute_handles, sync_handle_transforms, sync_handle_visibility,
    chute_handle_drag_system, draw_chute_gizmos,
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
        .init_resource::<AllMarbleRuns>()
        .add_systems(
            Startup,
            (setup_system, setup_snare_sound, setup_axes_hud, setup_chute_handles, setup_marble_trail_assets),
        )
        .add_systems(
            Update,
            (
                chute_handle_drag_system,
                spawn_marble_on_click_system.after(chute_handle_drag_system),
                orbit_camera_system,
                despawn_fallen_marbles_system,
                update_axes_hud,
                resize_axes_viewport,
                snare_hit_sound_system,
                trail_record_system,
                track_slide_end_system,
                record_snare_aoa_system,
                record_chute_marble_system,
                hud_panel_ui,
                chute_editor_ui,
                rebuild_chute_system,
                marble_graph_ui,
                update_marble_collisions,
                sync_handle_transforms,
                sync_handle_visibility,
                draw_chute_gizmos,
            ),
        )
        .run();
}
