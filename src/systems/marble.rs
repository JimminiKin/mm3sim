use bevy::math::primitives::Sphere;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::resources::constants::*;

pub fn spawn_marble_on_click_system(
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let mut rng = rand::thread_rng();
    let spawn_position = Vec3::new(
        MARBLE_SPAWN_X + rng.gen_range(-MARBLE_SPAWN_JITTER..MARBLE_SPAWN_JITTER),
        SPAWN_HEIGHT,
        rng.gen_range(-MARBLE_SPAWN_JITTER..MARBLE_SPAWN_JITTER),
    );

    spawn_marble(&mut commands, &mut meshes, &mut materials, spawn_position);
    spawn_chute_marble(&mut commands, &mut meshes, &mut materials);
}

#[derive(Component)]
pub struct Marble;

pub fn despawn_fallen_marbles_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Marble>>,
) {
    for (entity, transform) in &query {
        if transform.translation.y < DESPAWN_Y {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn spawn_marble(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    commands.spawn((
        Marble,
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere {
                radius: MARBLE_RADIUS,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(MARBLE_COLOR.0, MARBLE_COLOR.1, MARBLE_COLOR.2),
                metallic: MARBLE_METALLIC,
                perceptual_roughness: MARBLE_ROUGHNESS,
                ..default()
            }),
            transform: Transform::from_translation(position),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(MARBLE_RADIUS),
        ColliderMassProperties::Mass(MARBLE_MASS),
        Restitution::coefficient(STEEL_RESTITUTION),
        Friction::coefficient(STEEL_FRICTION),
        ActiveEvents::COLLISION_EVENTS,
        GravityScale::default(),
        Velocity::default(),
    ));
}

#[derive(Component)]
pub struct ChuteMarble;

pub fn spawn_chute_marble(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let position = Vec3::new(CHUTE_END_X, CHUTE_START_Y, CHUTE_START_Z - MARBLE_RADIUS);
    commands.spawn((
        Marble,
        ChuteMarble,
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere {
                radius: MARBLE_RADIUS,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(
                    CHUTE_MARBLE_COLOR.0,
                    CHUTE_MARBLE_COLOR.1,
                    CHUTE_MARBLE_COLOR.2,
                ),
                metallic: MARBLE_METALLIC,
                perceptual_roughness: MARBLE_ROUGHNESS,
                ..default()
            }),
            transform: Transform::from_translation(position),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(MARBLE_RADIUS),
        ColliderMassProperties::Mass(MARBLE_MASS),
        Restitution::coefficient(STEEL_RESTITUTION),
        Friction::coefficient(STEEL_FRICTION),
        ActiveEvents::COLLISION_EVENTS,
        GravityScale::default(),
        Velocity::default(),
    ));
}
