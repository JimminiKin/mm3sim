use bevy::math::primitives::Cuboid;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::resources::constants::*;

pub fn spawn_cycloid_chute(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    use std::f32::consts::PI;

    let r = CHUTE_R;

    let mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.55, 0.45, 0.30),
        metallic: 0.0,
        perceptual_roughness: 0.65,
        ..default()
    });

    let n = CHUTE_SEGMENTS;
    for i in 0..n - 12 {
        let t0 = (i as f32 / n as f32) * PI;
        let t1 = ((i + 1) as f32 / n as f32) * PI;

        // Chute runs along Z: high Z (toward pivot) → low Z (toward snare center)
        let z0 = CHUTE_START_Z - r * (t0 - t0.sin());
        let y0 = CHUTE_START_Y - r * (1.0 - t0.cos());
        let z1 = CHUTE_START_Z - r * (t1 - t1.sin());
        let y1 = CHUTE_START_Y - r * (1.0 - t1.cos());

        let mz = (z0 + z1) * 0.5;
        let my = (y0 + y1) * 0.5;
        let dz = z1 - z0;
        let dy = y1 - y0;
        let len = (dz * dz + dy * dy).sqrt();

        // Align the cuboid's Z axis with the segment direction (in Y-Z plane).
        // X axis (CHUTE_WIDTH) stays horizontal, spanning perpendicular to motion.
        let dir = Vec3::new(0.0, dy, dz).normalize();
        let rotation = Quat::from_rotation_arc(Vec3::Z, dir);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(Cuboid::new(CHUTE_WIDTH, CHUTE_THICKNESS, len))),
                material: mat.clone(),
                transform: Transform::from_xyz(CHUTE_END_X, my, mz).with_rotation(rotation),
                ..default()
            },
            RigidBody::Fixed,
            Collider::cuboid(CHUTE_WIDTH * 0.5, CHUTE_THICKNESS * 0.5, len * 0.5),
        ));
    }
}
