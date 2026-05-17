use bevy::math::primitives::Sphere;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::resources::marble_collisions::MarbleCollisions;
use crate::systems::chute_handles::HandleDrag;

/// Marbles live in GROUP_1. Snare/chute use the rapier default (ALL).
/// When marble-marble collisions are off, filter only matches GROUP_2 (snare/chute),
/// so marbles pass through each other while still hitting the snare.
fn marble_filter(collide: bool) -> Group {
    if collide {
        Group::GROUP_1 | Group::GROUP_2
    } else {
        Group::GROUP_2
    }
}

pub fn spawn_marble_on_click_system(
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chute_params: Res<ChuteParams>,
    marble_col: Res<MarbleCollisions>,
    drag: Res<HandleDrag>,
) {
    // Don't spawn when the user is dragging a Bézier handle
    if drag.active.is_some() {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let mut rng = rand::thread_rng();
    let spawn_position = Vec3::new(
        MARBLE_SPAWN_X + rng.gen_range(-MARBLE_SPAWN_JITTER..MARBLE_SPAWN_JITTER),
        SPAWN_HEIGHT,
        rng.gen_range(-MARBLE_SPAWN_JITTER..MARBLE_SPAWN_JITTER),
    );

    spawn_marble(&mut commands, &mut meshes, &mut materials, spawn_position, marble_col.0);
    spawn_chute_marble(&mut commands, &mut meshes, &mut materials, &chute_params, marble_col.0);
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

pub fn update_marble_collisions(
    settings: Res<MarbleCollisions>,
    mut marbles: Query<&mut CollisionGroups, With<Marble>>,
) {
    if !settings.is_changed() {
        return;
    }
    let filter = marble_filter(settings.0);
    for mut groups in &mut marbles {
        groups.filters = filter;
    }
}

pub fn spawn_marble(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    marble_marble_collide: bool,
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
        CollisionGroups::new(Group::GROUP_1, marble_filter(marble_marble_collide)),
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
    chute_params: &ChuteParams,
    marble_marble_collide: bool,
) {
    let position = Vec3::new(
        CHUTE_END_X,
        chute_params.p0[1],
        chute_params.p0[0] - MARBLE_RADIUS,
    );
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
        CollisionGroups::new(Group::GROUP_1, marble_filter(marble_marble_collide)),
        GravityScale::default(),
        Velocity::default(),
    ));
}
