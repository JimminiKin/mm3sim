use bevy::prelude::*;
use bevy::math::primitives::Cuboid;
use bevy_rapier3d::prelude::*;

use crate::resources::constants::*;

pub fn spawn_vibraphone_bar(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(BAR_LENGTH, BAR_HEIGHT, BAR_WIDTH))),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.85, 0.83, 0.65),
                metallic: 0.8,
                perceptual_roughness: 0.25,
                ..default()
            }),
            transform: Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_z(BAR_TILT)),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(BAR_LENGTH / 2.0, BAR_HEIGHT / 2.0, BAR_WIDTH / 2.0),
        Restitution::coefficient(0.15),
        Friction::coefficient(0.6),
    ));
}
