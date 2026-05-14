use bevy::math::primitives::Cylinder;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::resources::constants::*;

pub fn spawn_snare(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let chrome = materials.add(StandardMaterial {
        base_color: Color::rgb(CHROME_COLOR.0, CHROME_COLOR.1, CHROME_COLOR.2),
        metallic: CHROME_METALLIC,
        perceptual_roughness: CHROME_ROUGHNESS,
        ..default()
    });
    let dark_steel = materials.add(StandardMaterial {
        base_color: Color::rgb(DARK_STEEL_COLOR.0, DARK_STEEL_COLOR.1, DARK_STEEL_COLOR.2),
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
        .build();

    commands
        .spawn((
            TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, ARM_CENTER_Z)),
            RigidBody::Dynamic,
            Damping {
                linear_damping: ARM_LINEAR_DAMPING,
                angular_damping: ARM_ANGULAR_DAMPING,
            },
            ImpulseJoint::new(anchor, joint),
        ))
        .with_children(|p| {
            p.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(Cylinder {
                    radius: ARM_TUBE_RADIUS,
                    half_height: ARM_HALF_LEN,
                })),
                material: dark_steel.clone(),
                transform: Transform::from_rotation(Quat::from_rotation_x(
                    -std::f32::consts::FRAC_PI_2,
                )),
                ..default()
            });

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
                Collider::cylinder(SNARE_HALF_HEIGHT, SNARE_RADIUS),
                ColliderMassProperties::Mass(SNARE_MASS),
                Restitution::coefficient(STEEL_RESTITUTION),
                Friction::coefficient(STEEL_FRICTION),
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

            // Bumper: contact surface that rests on the stop post at 20° downward tilt
            p.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(Cylinder {
                        radius: STOP_BUMPER_RADIUS,
                        half_height: STOP_BUMPER_HALF_HEIGHT,
                    })),
                    material: dark_steel.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, STOP_BUMPER_LOCAL_Z),
                    ..default()
                },
                Collider::cylinder(STOP_BUMPER_HALF_HEIGHT, STOP_BUMPER_RADIUS),
            ));
        });

    // Fixed stop post: top surface meets the bumper bottom when arm tilts 20° snare-side down
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cylinder {
                radius: STOP_POST_RADIUS,
                half_height: STOP_POST_HALF_HEIGHT,
            })),
            material: dark_steel,
            transform: Transform::from_xyz(0.0, STOP_POST_Y, STOP_POST_Z),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cylinder(STOP_POST_HALF_HEIGHT, STOP_POST_RADIUS),
    ));
}
