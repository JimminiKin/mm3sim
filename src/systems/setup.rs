use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy_egui::PrimaryEguiContext;

use crate::components::chute::spawn_chute;
use crate::components::snare::spawn_snare;
use crate::components::vibraphone::spawn_vibraphone;
use crate::resources::chute_params::{ChuteParams, MultiChuteConfig, N_CHUTES};
use crate::resources::snare_params::SnareParams;
use crate::resources::vibraphone_params::VibraphoneParams;
use crate::resources::constants::*;
use crate::systems::camera::OrbitCamera;

pub fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chute_params: Res<ChuteParams>,
    multi_config: Res<MultiChuteConfig>,
    snare_params: Res<SnareParams>,
    vib_params: Res<VibraphoneParams>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::default(),
        OrbitCamera::default(),
        RenderLayers::layer(0),
        IsDefaultUiCamera,
        PrimaryEguiContext,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: LIGHT_ILLUMINANCE,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            LIGHT_ROT_X,
            LIGHT_ROT_Y,
            LIGHT_ROT_Z,
        )),
    ));

    commands.spawn(AmbientLight {
        brightness: AMBIENT_BRIGHTNESS,
        ..default()
    });

    spawn_snare(&mut commands, &mut meshes, &mut materials, &snare_params);
    for i in 0..N_CHUTES {
        let angle_rad = multi_config.angles_deg[i].to_radians();
        spawn_chute(&mut commands, &mut meshes, &mut materials, &chute_params, snare_params.pos, angle_rad);
    }
    spawn_vibraphone(&mut commands, &mut meshes, &mut materials, &vib_params);
}
