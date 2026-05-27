use avian3d::prelude::*;
use bevy::math::primitives::Cuboid;
use bevy::prelude::*;

use crate::components::instrument::{Instrument, MarbleSpawner};
use crate::components::pivot_arm::{spawn_pivot_arm, PivotArmSpec};
use crate::resources::constants::*;
use crate::resources::programming_wheel_params::WHEEL_CH_VIB_FIRST;
use crate::resources::vibraphone_params::VibraphoneParams;

#[derive(Component)]
pub struct VibraphoneBar {
    #[allow(dead_code)]
    pub index: u32,
}

#[derive(Component)]
pub struct VibraphoneArm;

/// Tags every top-level entity belonging to the vibraphone so that
/// `rebuild_vibraphone_system` can despawn the whole assembly.
///
/// Tagged on: anchor, arm, joint (per bar) and the standalone marble spawner.
/// Arm children (tube, CW, bar) are removed via Bevy's hierarchical despawn.
#[derive(Component, Clone)]
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

    // Bar centre Y (pos.y is top face; bar centre is half-thickness below).
    let bar_center_y = params.pos.y - params.bar_thickness * 0.5;
    let bar_center_z = params.pos.z;

    for bar_idx in 0..bar_count {
        // Bar length decreases per semitone: L ∝ 2^(−i/24) equal-temperament scaling.
        let bar_length = (params.bar_length_max * 2.0_f32.powf(-(bar_idx as f32) / 24.0))
            .max(params.bar_length_min);

        // Realistic bar mass: aluminium density × actual bar volume.
        let bar_mass = params.bar_density * params.bar_width * params.bar_thickness * bar_length;

        // Arm and pivot dimensions scale with bar length.
        let arm_half = bar_length * params.arm_scale * 0.5;
        let pivot_from_bar = bar_length * params.pivot_frac;
        let pivot_local_z = pivot_from_bar - arm_half; // offset from arm centre toward CW
        let cw_distance = arm_half - pivot_local_z;    // = bar_length × (arm_scale − pivot_frac)
        let cw_mass = (bar_mass * pivot_from_bar + VIB_ARM_MASS * pivot_local_z)
            / cw_distance
            * params.cw_weight_ratio;

        // X position: bars fan symmetrically around pos.x.
        let bar_x = params.pos.x
            + ((bar_count - 1) as f32 * 0.5 - bar_idx as f32) * params.bar_spacing;

        // Pivot world position derived from desired bar centre at rest angle.
        // D = arm_half + pivot_local_z = pivot_from_bar
        // pivot = (bar_x, bar_center_y − D·sin_r, bar_center_z + D·cos_r)
        let d = arm_half + pivot_local_z; // = pivot_from_bar
        let pivot_world = Vec3::new(
            bar_x,
            bar_center_y - d * sin_r,
            bar_center_z + d * cos_r,
        );

        // Scale CW cylinder proportionally to arm size.
        let cw_half_h = (arm_half * 0.12).max(VIB_CW_HALF_HEIGHT);
        let cw_r = (arm_half * 0.065).max(VIB_CW_RADIUS);

        let spec = PivotArmSpec {
            pivot_world_pos: pivot_world,
            arm_half_len: arm_half,
            pivot_local_z,
            rest_deg: params.rest_deg,
            max_tilt_deg: params.max_tilt_deg,
            arm_mass: VIB_ARM_MASS,
            arm_tube_radius: VIB_ARM_TUBE_RADIUS,
            linear_damping: VIB_LINEAR_DAMPING,
            angular_damping: params.angular_damping,
            cw_mass,
            cw_radius: cw_r,
            cw_half_height: cw_half_h,
            stand_half_height: 0.0, // vibraphone has no stand
        };

        // spawn_pivot_arm creates: anchor (VibraphoneEntity), arm (VibraphoneEntity) +
        // tube child + CW child, joint (VibraphoneEntity). No stand (0.0).
        let arm = spawn_pivot_arm(commands, meshes, &spec, frame_mat.clone(), VibraphoneEntity);

        // Tag the arm as VibraphoneArm and attach the bar at arm local z = −arm_half.
        commands.entity(arm).insert(VibraphoneArm).with_children(|p| {
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
                Instrument { channel: WHEEL_CH_VIB_FIRST + bar_idx as usize },
            ));
        });

        // Standalone marble spawn-point marker for this bar.
        // `sync_instrument_spawners` keeps it positioned above the bar each frame.
        // Tagged VibraphoneEntity so it is despawned on vibraphone rebuild.
        commands.spawn((
            Transform::default(),
            MarbleSpawner { channel: WHEEL_CH_VIB_FIRST + bar_idx as usize },
            VibraphoneEntity,
        ));
    }
}
