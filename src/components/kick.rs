use avian3d::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;

use crate::components::instrument::Instrument;
use crate::components::pivot_arm::{spawn_pivot_arm, PivotArmSpec};
use crate::resources::constants::*;
use crate::resources::kick_params::KickParams;
use crate::resources::programming_wheel_params::WHEEL_CH_KICK_FIRST;

/// Tags every entity that belongs to the kick drum assembly.
#[derive(Component, Clone)]
pub struct KickPart;

pub fn spawn_kick(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: &KickParams,
) {
    let drum_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(KICK_COLOR.0, KICK_COLOR.1, KICK_COLOR.2),
        metallic: KICK_METALLIC,
        perceptual_roughness: KICK_ROUGHNESS,
        ..default()
    });
    let frame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(DARK_STEEL_COLOR.0, DARK_STEEL_COLOR.1, DARK_STEEL_COLOR.2),
        metallic: DARK_STEEL_METALLIC,
        perceptual_roughness: DARK_STEEL_ROUGHNESS,
        ..default()
    });

    let rest_rad = params.rest_deg.to_radians();
    let pivot_world_pos = Vec3::new(
        params.pos.x,
        params.pos.y + KICK_PIVOT_FROM_DRUM * rest_rad.sin(),
        params.pos.z + KICK_PIVOT_FROM_DRUM * rest_rad.cos(),
    );

    let cw_mass = (KICK_MASS * KICK_PIVOT_FROM_DRUM + KICK_ARM_MASS * KICK_PIVOT_LOCAL_Z)
        / KICK_CW_DISTANCE
        * params.cw_weight_ratio;

    let spec = PivotArmSpec {
        pivot_world_pos,
        arm_half_len: KICK_ARM_HALF_LEN,
        pivot_local_z: KICK_PIVOT_LOCAL_Z,
        rest_deg: params.rest_deg,
        max_tilt_deg: params.max_tilt_deg,
        arm_mass: KICK_ARM_MASS,
        arm_tube_radius: KICK_ARM_TUBE_RADIUS,
        linear_damping: 0.0,
        angular_damping: params.angular_damping,
        cw_mass,
        cw_radius: KICK_CW_RADIUS,
        cw_half_height: KICK_CW_HALF_HEIGHT,
        stand_half_height: 0.0,
    };

    let arm = spawn_pivot_arm(commands, meshes, &spec, frame_mat, KickPart);

    commands.entity(arm).with_children(|p| {
        p.spawn((
            Mesh3d(meshes.add(Mesh::from(Cylinder {
                radius: KICK_RADIUS,
                half_height: KICK_HALF_HEIGHT,
            }))),
            MeshMaterial3d(drum_mat),
            Transform::from_xyz(0.0, 0.0, -KICK_ARM_HALF_LEN),
            Collider::cylinder(KICK_RADIUS, KICK_HALF_HEIGHT * 2.0),
            Mass(KICK_MASS),
            Restitution::new(params.restitution),
            Friction::new(params.friction),
            CollisionEventsEnabled,
            Instrument { channel: WHEEL_CH_KICK_FIRST },
        ));
    });
}
