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

        let x0 = CHUTE_START_X + r * (t0 - t0.sin());
        let y0 = CHUTE_START_Y - r * (1.0 - t0.cos());
        let x1 = CHUTE_START_X + r * (t1 - t1.sin());
        let y1 = CHUTE_START_Y - r * (1.0 - t1.cos());

        let mx = (x0 + x1) * 0.5;
        let my = (y0 + y1) * 0.5;
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(Cuboid::new(len, CHUTE_THICKNESS, CHUTE_WIDTH))),
                material: mat.clone(),
                transform: Transform::from_xyz(mx, my, CHUTE_END_Z)
                    .with_rotation(Quat::from_rotation_z(angle)),
                ..default()
            },
            RigidBody::Fixed,
            Collider::cuboid(len * 0.5, CHUTE_THICKNESS * 0.5, CHUTE_WIDTH * 0.5),
        ));
    }
}
