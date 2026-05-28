use avian3d::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;

use crate::components::instrument::Instrument;
use crate::resources::constants::*;
use crate::resources::programming_wheel_params::WHEEL_CH_RIDE_FIRST;

/// Tags every entity that belongs to the ride cymbal assembly.
#[derive(Component)]
pub struct RidePart;

pub fn spawn_ride(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let tilt = Quat::from_rotation_x(ARM_SPAWN_DEG.to_radians());
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(RIDE_COLOR.0, RIDE_COLOR.1, RIDE_COLOR.2),
        metallic: RIDE_METALLIC,
        perceptual_roughness: RIDE_ROUGHNESS,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cylinder {
            radius: RIDE_RADIUS,
            half_height: RIDE_HALF_HEIGHT,
        }))),
        MeshMaterial3d(mat),
        Transform {
            translation: Vec3::new(RIDE_X, RIDE_Y, RIDE_Z),
            rotation: tilt,
            scale: Vec3::ONE,
        },
        RigidBody::Static,
        Collider::cylinder(RIDE_RADIUS, RIDE_HALF_HEIGHT * 2.0),
        Restitution::new(RIDE_RESTITUTION),
        Friction::new(RIDE_FRICTION),
        CollisionEventsEnabled,
        Instrument { channel: WHEEL_CH_RIDE_FIRST },
        RidePart,
    ));
}
