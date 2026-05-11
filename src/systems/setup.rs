use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::vibraphone::spawn_vibraphone_bar;
use crate::resources::constants::*;

pub fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 8.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 25_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.7, 0.0)),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        brightness: 0.35,
        ..default()
    });

    spawn_vibraphone_bar(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 0.85, 0.0),
    );
}