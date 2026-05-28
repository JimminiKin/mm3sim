use avian3d::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;

use crate::components::instrument::Instrument;
use crate::components::pivot_arm::{spawn_pivot_arm, PivotArmSpec};
use crate::resources::constants::*;
use crate::resources::programming_wheel_params::WHEEL_CH_DROP;
use crate::resources::snare_params::SnareParams;

/// Marks the snare drum surface (hittable by marbles).
#[derive(Component)]
pub struct SnareDrum;

/// Marks the pivot arm entity for the snare (used by `apply_snare_fixed_system`).
#[derive(Component)]
pub struct PivotArm;

/// Tags every top-level entity that belongs to the snare assembly so that
/// `rebuild_snare_system` can despawn the whole thing.
///
/// The arm's children (tube, CW, drum) are removed automatically via Bevy's
/// hierarchical despawn — they do not need this marker themselves.
#[derive(Component, Clone)]
pub struct SnarePart;

pub fn spawn_snare(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: &SnareParams,
) {
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

    // Pivot anchor position: offset from world origin by params.pos so the
    // entire assembly moves as a rigid unit when params.pos changes.
    let spec = PivotArmSpec {
        pivot_world_pos: Vec3::new(0.0, 0.0, PIVOT_FROM_SNARE) + params.pos,
        arm_half_len: ARM_HALF_LEN,
        pivot_local_z: PIVOT_LOCAL_Z,
        rest_deg: SNARE_REST_DEG,
        max_tilt_deg: MAX_TILT_DEG,
        arm_mass: ARM_MASS,
        arm_tube_radius: ARM_TUBE_RADIUS,
        linear_damping: ARM_LINEAR_DAMPING,
        angular_damping: ARM_ANGULAR_DAMPING,
        cw_mass: CW_MASS,
        cw_radius: CW_RADIUS,
        cw_half_height: CW_HALF_HEIGHT,
        stand_half_height: 0.0,
    };

    // spawn_pivot_arm creates: anchor (SnarePart), stand (SnarePart),
    // arm (SnarePart) + tube child + CW child, joint (SnarePart).
    let arm = spawn_pivot_arm(commands, meshes, &spec, dark_steel, SnarePart);

    // Tag the arm as PivotArm and attach the snare drum surface at the
    // instrument end (local z = SNARE_LOCAL_Z = −ARM_HALF_LEN).
    commands.entity(arm).insert(PivotArm).with_children(|p| {
        p.spawn((
            Mesh3d(meshes.add(Mesh::from(Cylinder {
                radius: SNARE_RADIUS,
                half_height: SNARE_HALF_HEIGHT,
            }))),
            MeshMaterial3d(chrome),
            Transform::from_xyz(0.0, 0.0, SNARE_LOCAL_Z),
            Collider::cylinder(SNARE_RADIUS, SNARE_HALF_HEIGHT * 2.0),
            Mass(SNARE_MASS),
            Restitution::new(params.restitution),
            Friction::new(params.friction),
            CollisionEventsEnabled,
            SnareDrum,
            Instrument { channel: WHEEL_CH_DROP },
        ));
    });
}
