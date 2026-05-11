use bevy::prelude::*;
use bevy::math::primitives::Sphere;
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
        PLATE_LEFT_X + PLATE_WIDTH / 3.0 + rng.gen_range(-MARBLE_SPAWN_JITTER..MARBLE_SPAWN_JITTER),
        SPAWN_HEIGHT,
        rng.gen_range(-MARBLE_SPAWN_JITTER..MARBLE_SPAWN_JITTER),
    );

    spawn_marble(&mut commands, &mut meshes, &mut materials, spawn_position);
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
            mesh: meshes.add(Mesh::from(Sphere { radius: MARBLE_RADIUS })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.95, 0.35, 0.15),
                metallic: 0.8,
                perceptual_roughness: 0.2,
                ..default()
            }),
            transform: Transform::from_translation(position),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(MARBLE_RADIUS),
        Restitution::coefficient(0.75),
        Friction::coefficient(0.6),
        GravityScale(1.0),
        Velocity::default(),
    ));
}