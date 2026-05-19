use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy_egui::PrimaryEguiContext;

use crate::components::chute::spawn_chute;
use crate::components::snare::spawn_snare;
use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::systems::camera::OrbitCamera;

pub fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chute_params: Res<ChuteParams>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(CAMERA_POS.0, CAMERA_POS.1, CAMERA_POS.2)
            .looking_at(Vec3::ZERO, Vec3::Y),
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

    spawn_snare(&mut commands, &mut meshes, &mut materials);
    spawn_chute(&mut commands, &mut meshes, &mut materials, &chute_params);
}
