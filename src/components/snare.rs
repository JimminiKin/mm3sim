use avian3d::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;

use crate::resources::constants::*;

#[derive(Component)]
pub struct SnareDrum;

#[derive(Component)]
pub struct PivotArm;

pub fn spawn_snare(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
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
            Transform::from_xyz(0.0, 0.0, PIVOT_FROM_SNARE),
            RigidBody::Static,
        ))
        .id();

    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cylinder {
            radius: ARM_TUBE_RADIUS,
            half_height: PIVOT_STAND_HALF_HEIGHT,
        }))),
        MeshMaterial3d(dark_steel.clone()),
        Transform::from_xyz(0.0, -PIVOT_STAND_HALF_HEIGHT, PIVOT_FROM_SNARE),
    ));

    let arm = commands
        .spawn((
            Transform::from_xyz(0.0, arm_spawn_y, arm_spawn_z)
                .with_rotation(Quat::from_rotation_x(arm_rad)),
            Visibility::default(),
            RigidBody::Dynamic,
            PivotArm,
            LinearDamping(ARM_LINEAR_DAMPING),
            AngularDamping(ARM_ANGULAR_DAMPING),
        ))
        .with_children(|p| {
            p.spawn((
                Mesh3d(meshes.add(Mesh::from(Cylinder {
                    radius: ARM_TUBE_RADIUS,
                    half_height: ARM_HALF_LEN,
                }))),
                MeshMaterial3d(dark_steel.clone()),
                Transform::from_rotation(Quat::from_rotation_x(
                    -std::f32::consts::FRAC_PI_2,
                )),
                Collider::cylinder(ARM_TUBE_RADIUS, ARM_HALF_LEN * 2.0),
                Mass(ARM_MASS),
            ));

            p.spawn((
                Mesh3d(meshes.add(Mesh::from(Cylinder {
                    radius: SNARE_RADIUS,
                    half_height: SNARE_HALF_HEIGHT,
                }))),
                MeshMaterial3d(chrome),
                Transform::from_xyz(0.0, 0.0, SNARE_LOCAL_Z),
                Collider::cylinder(SNARE_RADIUS, SNARE_HALF_HEIGHT * 2.0),
                Mass(SNARE_MASS),
                Restitution::new(SNARE_RESTITUTION),
                Friction::new(SNARE_FRICTION),
                CollisionEventsEnabled,
                SnareDrum,
            ));

            p.spawn((
                Mesh3d(meshes.add(Mesh::from(Cylinder {
                    radius: CW_RADIUS,
                    half_height: CW_HALF_HEIGHT,
                }))),
                MeshMaterial3d(dark_steel.clone()),
                Transform::from_xyz(0.0, 0.0, CW_LOCAL_Z),
                Collider::cylinder(CW_RADIUS, CW_HALF_HEIGHT * 2.0),
                Mass(CW_MASS),
            ));
        })
        .id();

    commands.spawn(
        RevoluteJoint::new(anchor, arm)
            .with_hinge_axis(Vec3::X)
            .with_local_anchor1(Vec3::ZERO)
            .with_local_anchor2(Vec3::new(0.0, 0.0, PIVOT_LOCAL_Z))
            .with_angle_limits(
                -(SNARE_REST_DEG + MAX_TILT_DEG).to_radians(),
                -SNARE_REST_DEG.to_radians(),
            ),
    );
}
