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
    let top_thickness = BAR_HEIGHT - BAR_CUTOUT_DEPTH;
    let side_width = (BAR_WIDTH - BAR_CUTOUT_WIDTH) / 2.0;
    let side_offset_z = (BAR_WIDTH + BAR_CUTOUT_WIDTH) / 4.0;

    let bar_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.85, 0.83, 0.65),
        metallic: 0.8,
        perceptual_roughness: 0.25,
        ..default()
    });

    let top_mesh = meshes.add(Mesh::from(Cuboid::new(
        BAR_LENGTH,
        top_thickness,
        BAR_WIDTH,
    )));
    let support_mesh = meshes.add(Mesh::from(Cuboid::new(
        BAR_LENGTH,
        BAR_CUTOUT_DEPTH,
        side_width,
    )));

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_translation(position)
                    .with_rotation(Quat::from_rotation_z(BAR_TILT)),
                ..default()
            },
            RigidBody::Fixed,
            Collider::compound(vec![
                (
                    Vec3::new(0.0, BAR_CUTOUT_DEPTH / 2.0, 0.0),
                    Quat::IDENTITY,
                    Collider::cuboid(
                        BAR_LENGTH / 2.0,
                        top_thickness / 2.0,
                        BAR_WIDTH / 2.0,
                    ),
                ),
                (
                    Vec3::new(
                        0.0,
                        -BAR_HEIGHT / 2.0 + BAR_CUTOUT_DEPTH / 2.0,
                        -side_offset_z,
                    ),
                    Quat::IDENTITY,
                    Collider::cuboid(
                        BAR_LENGTH / 2.0,
                        BAR_CUTOUT_DEPTH / 2.0,
                        side_width / 2.0,
                    ),
                ),
                (
                    Vec3::new(
                        0.0,
                        -BAR_HEIGHT / 2.0 + BAR_CUTOUT_DEPTH / 2.0,
                        side_offset_z,
                    ),
                    Quat::IDENTITY,
                    Collider::cuboid(
                        BAR_LENGTH / 2.0,
                        BAR_CUTOUT_DEPTH / 2.0,
                        side_width / 2.0,
                    ),
                ),
            ]),
            Restitution::coefficient(0.15),
            Friction::coefficient(0.6),
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: top_mesh.clone(),
                material: bar_material.clone(),
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    BAR_CUTOUT_DEPTH / 2.0,
                    0.0,
                )),
                ..default()
            });

            parent.spawn(PbrBundle {
                mesh: support_mesh.clone(),
                material: bar_material.clone(),
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    -BAR_HEIGHT / 2.0 + BAR_CUTOUT_DEPTH / 2.0,
                    -side_offset_z,
                )),
                ..default()
            });

            parent.spawn(PbrBundle {
                mesh: support_mesh,
                material: bar_material,
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    -BAR_HEIGHT / 2.0 + BAR_CUTOUT_DEPTH / 2.0,
                    side_offset_z,
                )),
                ..default()
            });
        });
}