use bevy::math::primitives::{Cuboid, Cylinder};
use bevy::prelude::*;

use crate::resources::constants::*;

#[derive(Component)]
pub struct ProgrammingWheelCylinder;

#[derive(Component)]
pub struct ProgrammingWheelReaderBar;

/// Marks every entity that belongs to the programming wheel so they can all be despawned on rebuild.
#[derive(Component)]
pub struct ProgrammingWheelEntity;

pub fn spawn_programming_wheel(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Semi-transparent cylinder
    let cylinder_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.45, 0.60, 0.80, 0.30),
        metallic: 0.30,
        perceptual_roughness: 0.55,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    // Bright golden reader bar
    let reader_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.80, 0.10),
        metallic: 0.85,
        perceptual_roughness: 0.15,
        ..default()
    });

    // Cylinder: Bevy's Cylinder has its axis along Y.
    // Rotate 90° around Z so the axis aligns with world X.
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cylinder {
            radius: PROGRAMMING_WHEEL_RADIUS,
            half_height: PROGRAMMING_WHEEL_WIDTH * 0.5,
        }))),
        MeshMaterial3d(cylinder_mat),
        Transform::from_xyz(0.0, PROGRAMMING_WHEEL_Y_POS, PROGRAMMING_WHEEL_Z_POS)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ProgrammingWheelCylinder,
        ProgrammingWheelEntity,
    ));

    // Reader bar — thin rectangular bar just above the cylinder top
    let reader_y = PROGRAMMING_WHEEL_Y_POS
        + PROGRAMMING_WHEEL_RADIUS
        + PROGRAMMING_WHEEL_READER_GAP
        + PROGRAMMING_WHEEL_READER_HALF_H;
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cuboid {
            half_size: Vec3::new(
                PROGRAMMING_WHEEL_WIDTH * 0.5 + 0.025,
                PROGRAMMING_WHEEL_READER_HALF_H,
                PROGRAMMING_WHEEL_READER_HALF_H,
            ),
        }))),
        MeshMaterial3d(reader_mat),
        Transform::from_xyz(0.0, reader_y, PROGRAMMING_WHEEL_Z_POS),
        ProgrammingWheelReaderBar,
        ProgrammingWheelEntity,
    ));
}
