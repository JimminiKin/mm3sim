use bevy::math::primitives::Cylinder;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::resources::constants::*;

/// Flat disk trimesh at the snare top face (y = +SNARE_HALF_HEIGHT in local space).
/// A pure flat polygon gives Rapier an unambiguous face normal, avoiding the
/// GJK instability that comes from the cylinder's flat-cap / curved-side boundary.
fn snare_head_collider() -> Collider {
    const N: u32 = 48;
    let h = SNARE_HALF_HEIGHT;
    let r = SNARE_RADIUS;

    let mut verts: Vec<Vec3> = Vec::with_capacity((N + 1) as usize);
    verts.push(Vec3::new(0.0, h, 0.0)); // centre (index 0)
    for i in 0..N {
        let a = 2.0 * std::f32::consts::PI * i as f32 / N as f32;
        verts.push(Vec3::new(r * a.cos(), h, r * a.sin()));
    }

    // Winding that gives normal = +Y (toward incoming marble).
    // curr = rim vertex i+1, next = rim vertex (i+1)%N+1 (wraps 48→1).
    // Cross product of (next-center)×(curr-center) points +Y.
    let mut indices: Vec<[u32; 3]> = Vec::with_capacity(N as usize);
    for i in 0..N {
        let curr = i + 1;
        let next = (i + 1) % N + 1;
        indices.push([0, next, curr]);
    }

    Collider::trimesh_with_flags(
        verts,
        indices,
        TriMeshFlags::FIX_INTERNAL_EDGES | TriMeshFlags::MERGE_DUPLICATE_VERTICES,
    )
}

#[derive(Component)]
pub struct SnareDrum;

#[derive(Component)]
pub struct PivotArm;

pub fn spawn_snare(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Arm geometry — derived from degree constants
    let arm_rad = ARM_SPAWN_DEG.to_radians();
    let arm_spawn_y = PIVOT_LOCAL_Z * arm_rad.sin();
    let arm_spawn_z = PIVOT_FROM_SNARE - PIVOT_LOCAL_Z * arm_rad.cos();

    let chrome = materials.add(StandardMaterial {
        base_color: Color::srgb(CHROME_COLOR.0, CHROME_COLOR.1, CHROME_COLOR.2),
        metallic: CHROME_METALLIC,
        perceptual_roughness: CHROME_ROUGHNESS,
        ..default()
    });
    let dark_steel = materials.add(StandardMaterial {
        base_color: Color::srgb(DARK_STEEL_COLOR.0, DARK_STEEL_COLOR.1, DARK_STEEL_COLOR.2),
        metallic: DARK_STEEL_METALLIC,
        perceptual_roughness: DARK_STEEL_ROUGHNESS,
        ..default()
    });

    let anchor = commands
        .spawn((
            TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, PIVOT_FROM_SNARE)),
            RigidBody::Fixed,
        ))
        .id();

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cylinder {
            radius: ARM_TUBE_RADIUS,
            half_height: PIVOT_STAND_HALF_HEIGHT,
        })),
        material: dark_steel.clone(),
        transform: Transform::from_xyz(0.0, -PIVOT_STAND_HALF_HEIGHT, PIVOT_FROM_SNARE),
        ..default()
    });

    let joint = RevoluteJointBuilder::new(Vec3::X)
        .local_anchor1(Vec3::ZERO)
        .local_anchor2(Vec3::new(0.0, 0.0, PIVOT_LOCAL_Z))
        .limits([
            -(SNARE_REST_DEG + MAX_TILT_DEG).to_radians(),
            -SNARE_REST_DEG.to_radians(),
        ])
        .build();

    commands
        .spawn((
            TransformBundle::from_transform(
                Transform::from_xyz(0.0, arm_spawn_y, arm_spawn_z)
                    .with_rotation(Quat::from_rotation_x(arm_rad)),
            ),
            RigidBody::Dynamic,
            PivotArm,
            Damping {
                linear_damping: ARM_LINEAR_DAMPING,
                angular_damping: ARM_ANGULAR_DAMPING,
            },
            ImpulseJoint::new(anchor, joint),
            LockedAxes::default(),
        ))
        .with_children(|p| {
            p.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(Cylinder {
                        radius: ARM_TUBE_RADIUS,
                        half_height: ARM_HALF_LEN,
                    })),
                    material: dark_steel.clone(),
                    transform: Transform::from_rotation(Quat::from_rotation_x(
                        -std::f32::consts::FRAC_PI_2,
                    )),
                    ..default()
                },
                Collider::cylinder(ARM_HALF_LEN, ARM_TUBE_RADIUS),
                ColliderMassProperties::Mass(ARM_MASS),
            ));

            p.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(Cylinder {
                        radius: SNARE_RADIUS,
                        half_height: SNARE_HALF_HEIGHT,
                    })),
                    material: chrome,
                    transform: Transform::from_xyz(0.0, 0.0, SNARE_LOCAL_Z),
                    ..default()
                },
                snare_head_collider(),
                ColliderMassProperties::Mass(SNARE_MASS),
                Restitution::coefficient(SNARE_RESTITUTION),
                Friction::coefficient(SNARE_FRICTION),
                SnareDrum,
                ActiveEvents::COLLISION_EVENTS,
            ));

            p.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(Cylinder {
                        radius: CW_RADIUS,
                        half_height: CW_HALF_HEIGHT,
                    })),
                    material: dark_steel.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, CW_LOCAL_Z),
                    ..default()
                },
                Collider::cylinder(CW_HALF_HEIGHT, CW_RADIUS),
                ColliderMassProperties::Mass(CW_MASS),
            ));
        });

}
