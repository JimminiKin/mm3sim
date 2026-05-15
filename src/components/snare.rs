use bevy::math::primitives::Cylinder;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::resources::constants::*;

#[derive(Component)]
pub struct SnareDrum;

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
            TransformBundle::from_transform(
                Transform::from_xyz(0.0, ARM_SPAWN_Y, ARM_SPAWN_Z)
                    .with_rotation(Quat::from_rotation_x(ARM_SPAWN_ANGLE_RAD)),
            ),
            RigidBody::Dynamic,
            Damping {
                linear_damping: ARM_LINEAR_DAMPING,
                angular_damping: ARM_ANGULAR_DAMPING,
            },
            ImpulseJoint::new(anchor, joint),
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
                Collider::cylinder(SNARE_HALF_HEIGHT, SNARE_RADIUS),
                ColliderMassProperties::Mass(SNARE_MASS),
                Restitution::coefficient(STEEL_RESTITUTION),
                Friction::coefficient(STEEL_FRICTION),
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

    // Lower stop tube: arm tube bottom rests here at 20° snare-down
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cylinder {
                radius: STOP_TUBE_RADIUS,
                half_height: STOP_TUBE_HALF_LEN,
            })),
            material: dark_steel.clone(),
            transform: Transform::from_xyz(0.0, STOP_POST_Y, STOP_POST_Z)
                .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cylinder(STOP_TUBE_HALF_LEN, STOP_TUBE_RADIUS),
    ));

    // Upper stop tube: arm tube top rests here at 15° snare-down
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cylinder {
                radius: STOP_TUBE_RADIUS,
                half_height: STOP_TUBE_HALF_LEN,
            })),
            material: dark_steel,
            transform: Transform::from_xyz(0.0, STOP_UPPER_POST_Y, STOP_UPPER_POST_Z)
                .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cylinder(STOP_TUBE_HALF_LEN, STOP_TUBE_RADIUS),
    ));
}
