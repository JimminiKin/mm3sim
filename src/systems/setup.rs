use bevy::prelude::*;

use crate::components::cycloid_chute::spawn_cycloid_chute;
use crate::components::snare::spawn_snare;
use crate::resources::constants::*;
use crate::systems::camera::OrbitCamera;

pub fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(CAMERA_POS.0, CAMERA_POS.1, CAMERA_POS.2)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitCamera::default(),
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: LIGHT_ILLUMINANCE,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            LIGHT_ROT_X,
            LIGHT_ROT_Y,
            LIGHT_ROT_Z,
        )),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        brightness: AMBIENT_BRIGHTNESS,
        ..default()
    });

    spawn_snare(&mut commands, &mut meshes, &mut materials);
    spawn_cycloid_chute(&mut commands, &mut meshes, &mut materials);
}
