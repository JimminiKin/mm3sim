use avian3d::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;

use crate::components::instrument::Instrument;
use crate::resources::constants::*;
use crate::resources::kick_params::KickParams;
use crate::resources::programming_wheel_params::WHEEL_CH_KICK_FIRST;

/// Tags every entity that belongs to the kick drum assembly.
#[derive(Component)]
pub struct KickPart;

pub fn spawn_kick(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: &KickParams,
) {
    let tilt = Quat::from_rotation_x(ARM_SPAWN_DEG.to_radians());
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(KICK_COLOR.0, KICK_COLOR.1, KICK_COLOR.2),
        metallic: KICK_METALLIC,
        perceptual_roughness: KICK_ROUGHNESS,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cylinder {
            radius: KICK_RADIUS,
            half_height: KICK_HALF_HEIGHT,
        }))),
        MeshMaterial3d(mat),
        Transform {
            translation: params.pos,
            rotation: tilt,
            scale: Vec3::ONE,
        },
        RigidBody::Static,
        Collider::cylinder(KICK_RADIUS, KICK_HALF_HEIGHT * 2.0),
        Restitution::new(params.restitution),
        Friction::new(params.friction),
        CollisionEventsEnabled,
        Instrument { channel: WHEEL_CH_KICK_FIRST },
        KickPart,
    ));
}
