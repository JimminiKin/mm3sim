use avian3d::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;

use crate::components::instrument::Instrument;
use crate::resources::constants::*;
use crate::resources::hihat_params::HiHatParams;
use crate::resources::programming_wheel_params::{WHEEL_CH_HIHAT_FIRST, WHEEL_CH_HIHAT_PEDAL};

/// Tags every entity that belongs to the hi-hat assembly.
#[derive(Component)]
pub struct HiHatPart;

/// Tags the purely-visual top cymbal so its Y can be updated when state changes.
#[derive(Component)]
pub struct HiHatTopCymbal;

pub fn spawn_hihat(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: &HiHatParams,
    open: bool,
) {
    let tilt = Quat::from_rotation_x(ARM_SPAWN_DEG.to_radians());
    let bot_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(HIHAT_COLOR.0, HIHAT_COLOR.1, HIHAT_COLOR.2),
        metallic: HIHAT_METALLIC,
        perceptual_roughness: HIHAT_ROUGHNESS,
        ..default()
    });
    let top_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(
            HIHAT_COLOR.0 * 0.88,
            HIHAT_COLOR.1 * 0.88,
            HIHAT_COLOR.2 * 0.88,
        ),
        metallic: HIHAT_METALLIC,
        perceptual_roughness: HIHAT_ROUGHNESS + 0.05,
        ..default()
    });
    let pedal_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(DARK_STEEL_COLOR.0, DARK_STEEL_COLOR.1, DARK_STEEL_COLOR.2),
        metallic: DARK_STEEL_METALLIC,
        perceptual_roughness: DARK_STEEL_ROUGHNESS,
        ..default()
    });

    // Bottom cymbal — static hit surface.
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cylinder {
            radius: HIHAT_RADIUS,
            half_height: HIHAT_HALF_HEIGHT,
        }))),
        MeshMaterial3d(bot_mat),
        Transform { translation: params.pos, rotation: tilt, scale: Vec3::ONE },
        RigidBody::Static,
        Collider::cylinder(HIHAT_RADIUS, HIHAT_HALF_HEIGHT * 2.0),
        Restitution::new(params.restitution),
        Friction::new(params.friction),
        CollisionEventsEnabled,
        Instrument { channel: WHEEL_CH_HIHAT_FIRST },
        HiHatPart,
    ));

    // Top cymbal — visual indicator of open/closed state, no physics.
    let gap = if open { params.gap_open } else { params.gap_closed };
    let top_offset = tilt * Vec3::Y * (gap + HIHAT_HALF_HEIGHT * 2.0);
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cylinder {
            radius: HIHAT_RADIUS,
            half_height: HIHAT_HALF_HEIGHT,
        }))),
        MeshMaterial3d(top_mat),
        Transform {
            translation: params.pos + top_offset,
            rotation: tilt,
            scale: Vec3::ONE,
        },
        HiHatTopCymbal,
        HiHatPart,
    ));

    // Pedal trigger — small disc beside the cymbal.
    let pedal_pos = Vec3::new(
        params.pos.x - HIHAT_RADIUS - 0.06,
        params.pos.y,
        params.pos.z,
    );
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cylinder {
            radius: HIHAT_PEDAL_RADIUS,
            half_height: HIHAT_PEDAL_HALF_HEIGHT,
        }))),
        MeshMaterial3d(pedal_mat),
        Transform {
            translation: pedal_pos,
            rotation: tilt,
            scale: Vec3::ONE,
        },
        RigidBody::Static,
        Collider::cylinder(HIHAT_PEDAL_RADIUS, HIHAT_PEDAL_HALF_HEIGHT * 2.0),
        Restitution::new(STEEL_RESTITUTION),
        Friction::new(STEEL_FRICTION),
        CollisionEventsEnabled,
        Instrument { channel: WHEEL_CH_HIHAT_PEDAL },
        HiHatPart,
    ));
}
