use avian3d::prelude::*;
use bevy::math::primitives::{Cuboid, Cylinder};
use bevy::prelude::*;

use crate::resources::constants::*;
use crate::resources::vibraphone_params::VibraphoneParams;

#[derive(Component)]
pub struct VibraphoneBar {
    pub index: u32,
}

#[derive(Component)]
pub struct VibraphoneArm;

/// Marker for every entity belonging to the vibraphone (used for rebuild despawn).
#[derive(Component)]
pub struct VibraphoneEntity;

pub fn spawn_vibraphone(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: &VibraphoneParams,
) {
    let arm_rad = (-params.rest_deg).to_radians();
    let sin_r = arm_rad.sin();
    let cos_r = arm_rad.cos();

    let bar_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(VIB_BAR_COLOR.0, VIB_BAR_COLOR.1, VIB_BAR_COLOR.2),
        metallic: VIB_BAR_METALLIC,
        perceptual_roughness: VIB_BAR_ROUGHNESS,
        ..default()
    });
    let frame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(DARK_STEEL_COLOR.0, DARK_STEEL_COLOR.1, DARK_STEEL_COLOR.2),
        metallic: DARK_STEEL_METALLIC,
        perceptual_roughness: DARK_STEEL_ROUGHNESS,
        ..default()
    });

    let bar_count = VIB_BAR_COUNT;

    // Bar center Y (row_y is top face, same for all bars)
    let bar_center_y = params.row_y - params.bar_thickness * 0.5;
    let bar_center_z = params.row_z;

    for bar_idx in 0..bar_count {
        // Bar length decreases per semitone: L ∝ 2^(-i/24) for equal-temperament scaling
        let bar_length = (params.bar_length_max * 2.0_f32.powf(-(bar_idx as f32) / 24.0))
            .max(params.bar_length_min);

        // Realistic bar mass: aluminium density × actual bar volume
        let bar_mass = params.bar_density * params.bar_width * params.bar_thickness * bar_length;

        // Arm and pivot dimensions scale with bar length so the mechanism stays proportional
        let arm_half = bar_length * params.arm_scale * 0.5;
        let pivot_from_bar = bar_length * params.pivot_frac;
        let pivot_local_z = pivot_from_bar - arm_half;
        let cw_distance = arm_half - pivot_local_z; // = bar_length*(arm_scale - pivot_frac)
        let cw_mass = (bar_mass * pivot_from_bar + VIB_ARM_MASS * pivot_local_z)
            / cw_distance
            * params.cw_weight_ratio;

        // X position: centered on row_x_center
        let bar_x = params.row_x_center
            + ((bar_count - 1) as f32 * 0.5 - bar_idx as f32) * params.bar_spacing;

        // Arm spawn: R*(0,0,-arm_half) = (0, arm_half*sin_r, -arm_half*cos_r)
        // arm_spawn = bar_center - R*(0,0,-arm_half)
        // Bar center is always at (bar_x, bar_center_y, bar_center_z) regardless of arm_half.
        let arm_spawn_y = bar_center_y - arm_half * sin_r;
        let arm_spawn_z = bar_center_z + arm_half * cos_r;

        // Pivot anchor world position
        let pivot_world_y = arm_spawn_y - pivot_local_z * sin_r;
        let pivot_world_z = arm_spawn_z + pivot_local_z * cos_r;

        let anchor = commands
            .spawn((
                Transform::from_xyz(bar_x, pivot_world_y, pivot_world_z),
                RigidBody::Static,
                VibraphoneEntity,
            ))
            .id();

        // Scale CW cylinder proportionally to arm size
        let cw_half_h = (arm_half * 0.12).max(VIB_CW_HALF_HEIGHT);
        let cw_r = (arm_half * 0.065).max(VIB_CW_RADIUS);

        let arm = commands
            .spawn((
                Transform::from_xyz(bar_x, arm_spawn_y, arm_spawn_z)
                    .with_rotation(Quat::from_rotation_x(arm_rad)),
                Visibility::default(),
                RigidBody::Dynamic,
                VibraphoneArm,
                VibraphoneEntity,
                LinearDamping(VIB_LINEAR_DAMPING),
                AngularDamping(params.angular_damping),
            ))
            .with_children(|p| {
                // Thin arm tube (length scales with bar)
                p.spawn((
                    Mesh3d(meshes.add(Mesh::from(Cylinder {
                        radius: VIB_ARM_TUBE_RADIUS,
                        half_height: arm_half,
                    }))),
                    MeshMaterial3d(frame_mat.clone()),
                    Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    Collider::cylinder(VIB_ARM_TUBE_RADIUS, arm_half * 2.0),
                    Mass(VIB_ARM_MASS),
                ));

                // Vibraphone bar (cuboid) centered at arm local z = -arm_half
                // Bar is centered in Z so its center stays at bar_center_z in world space
                p.spawn((
                    Mesh3d(meshes.add(Mesh::from(Cuboid {
                        half_size: Vec3::new(
                            params.bar_width * 0.5,
                            params.bar_thickness * 0.5,
                            bar_length * 0.5,
                        ),
                    }))),
                    MeshMaterial3d(bar_mat.clone()),
                    Transform::from_xyz(0.0, 0.0, -arm_half),
                    Collider::cuboid(
                        params.bar_width * 0.5,
                        params.bar_thickness * 0.5,
                        bar_length * 0.5,
                    ),
                    Mass(bar_mass),
                    Restitution::new(params.restitution),
                    Friction::new(params.friction),
                    CollisionEventsEnabled,
                    VibraphoneBar { index: bar_idx },
                    VibraphoneEntity,
                ));

                // Counterweight at far end (local z = +arm_half), scaled to arm size
                p.spawn((
                    Mesh3d(meshes.add(Mesh::from(Cylinder {
                        radius: cw_r,
                        half_height: cw_half_h,
                    }))),
                    MeshMaterial3d(frame_mat.clone()),
                    Transform::from_xyz(0.0, 0.0, arm_half),
                    Collider::cylinder(cw_r, cw_half_h * 2.0),
                    Mass(cw_mass),
                ));
            })
            .id();

        commands.spawn((
            RevoluteJoint::new(anchor, arm)
                .with_hinge_axis(Vec3::X)
                .with_local_anchor1(Vec3::ZERO)
                .with_local_anchor2(Vec3::new(0.0, 0.0, pivot_local_z))
                .with_angle_limits(
                    -(params.rest_deg + params.max_tilt_deg).to_radians(),
                    -params.rest_deg.to_radians(),
                ),
            VibraphoneEntity,
        ));
    }
}
