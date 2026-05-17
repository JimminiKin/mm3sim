use bevy::math::primitives::Cuboid;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;

#[derive(Component)]
pub struct ChuteSegment;

fn bezier(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), t: f32) -> (f32, f32) {
    let u = 1.0 - t;
    (
        u*u*u*p0.0 + 3.0*u*u*t*p1.0 + 3.0*u*t*t*p2.0 + t*t*t*p3.0,
        u*u*u*p0.1 + 3.0*u*u*t*p1.1 + 3.0*u*t*t*p2.1 + t*t*t*p3.1,
    )
}

pub fn spawn_cycloid_chute(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: &ChuteParams,
) {
    let p0 = (params.p0[0],  params.p0[1]);
    let p1 = (params.cp1[0], params.cp1[1]);
    let p2 = (params.cp2[0], params.cp2[1]);
    let p3 = (params.p3[0],  params.p3[1]);

    let mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.55, 0.45, 0.30),
        metallic: 0.0,
        perceptual_roughness: 0.65,
        ..default()
    });

    let n = CHUTE_SEGMENTS;
    for i in 0..n {
        let t0 = i as f32 / n as f32;
        let t1 = (i + 1) as f32 / n as f32;

        let (z0, y0) = bezier(p0, p1, p2, p3, t0);
        let (z1, y1) = bezier(p0, p1, p2, p3, t1);

        let mz = (z0 + z1) * 0.5;
        let my = (y0 + y1) * 0.5;
        let dz = z1 - z0;
        let dy = y1 - y0;
        let len = (dz * dz + dy * dy).sqrt();

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
            ChuteSegment,
        ));
    }
}
