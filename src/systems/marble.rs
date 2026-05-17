use bevy::math::primitives::Sphere;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::components::snare::SnareDrum;
use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::resources::marble_collisions::MarbleCollisions;
use crate::systems::chute_handles::HandleDrag;

// ── Collision filter ──────────────────────────────────────────────────────────

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

// ── Marble components ─────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Marble;

#[derive(Component)]
pub struct ChuteMarble;

#[derive(Component)]
pub struct SpawnTime(pub f32);

/// Accumulated seconds since the last trail dot was emitted for this marble.
#[derive(Component)]
pub struct TrailTimer(pub f32);

/// Per-marble slide tracking: time and velocity when the marble left the chute surface.
#[derive(Component, Default)]
pub struct SlideData {
    pub end_time: Option<f32>,
    pub end_vel: Option<Vec3>,
}

// ── Trail assets (shared mesh + materials, created once at startup) ───────────

const TRAIL_DOT_RADIUS: f32 = MARBLE_RADIUS * 0.5;
const TRAIL_INTERVAL_S: f32 = 0.005;

#[derive(Resource)]
pub struct MarbleTrailAssets {
    pub mesh: Handle<Mesh>,
    pub drop_mat: Handle<StandardMaterial>,
    pub chute_mat: Handle<StandardMaterial>,
}

pub fn setup_marble_trail_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(Sphere {
        radius: TRAIL_DOT_RADIUS,
    }));
    let drop_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(MARBLE_COLOR.0, MARBLE_COLOR.1, MARBLE_COLOR.2, 0.55),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let chute_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(
            CHUTE_MARBLE_COLOR.0,
            CHUTE_MARBLE_COLOR.1,
            CHUTE_MARBLE_COLOR.2,
            0.55,
        ),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.insert_resource(MarbleTrailAssets {
        mesh,
        drop_mat,
        chute_mat,
    });
}

// ── Trail dot marker ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct TrailDot;

// ── Spawn ─────────────────────────────────────────────────────────────────────

pub fn spawn_marble_on_click_system(
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chute_params: Res<ChuteParams>,
    marble_col: Res<MarbleCollisions>,
    drag: Res<HandleDrag>,
    time: Res<Time>,
    trail_dots: Query<Entity, With<TrailDot>>,
    snare: Query<&GlobalTransform, With<SnareDrum>>,
) {
    if drag.active.is_some() {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    // Clear previous trajectory history
    for entity in &trail_dots {
        commands.entity(entity).despawn();
    }

    // Snare top-face centre in world space (correct for any arm angle).
    let snare_top_y = snare
        .get_single()
        .map(|gt| gt.translation().y + SNARE_HALF_HEIGHT)
        .unwrap_or(CHUTE_ORIGIN_Y);

    let spawn_time = time.elapsed_seconds();
    let mut rng = rand::thread_rng();
    let spawn_position = Vec3::new(
        MARBLE_SPAWN_X + rng.gen_range(-MARBLE_SPAWN_JITTER..MARBLE_SPAWN_JITTER),
        snare_top_y + SPAWN_HEIGHT,
        rng.gen_range(-MARBLE_SPAWN_JITTER..MARBLE_SPAWN_JITTER),
    );

    spawn_marble(
        &mut commands,
        &mut meshes,
        &mut materials,
        spawn_position,
        marble_col.0,
        spawn_time,
    );
    spawn_chute_marble(
        &mut commands,
        &mut meshes,
        &mut materials,
        &chute_params,
        marble_col.0,
        spawn_time,
    );
}

pub fn spawn_marble(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    marble_marble_collide: bool,
    spawn_time: f32,
) {
    commands.spawn((
        Marble,
        SpawnTime(spawn_time),
        TrailTimer(0.0),
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere {
                radius: MARBLE_RADIUS,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(MARBLE_COLOR.0, MARBLE_COLOR.1, MARBLE_COLOR.2),
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

pub fn spawn_chute_marble(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chute_params: &ChuteParams,
    marble_marble_collide: bool,
    spawn_time: f32,
) {
    let pts = chute_params.effective_pts();
    let p0 = pts[0]; // [z, y]
    let cp1 = pts[1]; // [z, y]

    // Surface normal at t=0: cross(X, tangent_3d) where tangent = (0, dy, dz)
    let dz = 3.0 * (cp1[0] - p0[0]);
    let dy = 3.0 * (cp1[1] - p0[1]);
    let normal = Vec3::new(0.0, -dz, dy).normalize_or_zero();

    let chute_centre = Vec3::new(CHUTE_END_X, p0[1] + CHUTE_ORIGIN_Y, p0[0] + CHUTE_ORIGIN_Z);
    // Embed 1 mm into the surface: at spawn velocity=0 so Rapier's speculative-contact
    // prediction sees no approaching velocity and skips contact for one step, letting
    // gravity act freely. A small penetration depth forces immediate resolution.
    let position = chute_centre + normal * (CHUTE_THICKNESS * 0.5 + MARBLE_RADIUS - 0.004);
    commands.spawn((
        Marble,
        ChuteMarble,
        SpawnTime(spawn_time),
        TrailTimer(0.0),
        SlideData::default(),
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere {
                radius: MARBLE_RADIUS,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(
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

// ── Trail recording ───────────────────────────────────────────────────────────

pub fn trail_record_system(
    mut commands: Commands,
    time: Res<Time>,
    mut marbles: Query<(&Transform, Option<&ChuteMarble>, &mut TrailTimer), With<Marble>>,
    assets: Option<Res<MarbleTrailAssets>>,
) {
    let Some(assets) = assets else { return };
    let dt = time.delta_seconds();

    for (transform, is_chute, mut timer) in &mut marbles {
        timer.0 += dt;
        if timer.0 < TRAIL_INTERVAL_S {
            continue;
        }
        timer.0 -= TRAIL_INTERVAL_S;

        let mat = if is_chute.is_some() {
            assets.chute_mat.clone()
        } else {
            assets.drop_mat.clone()
        };

        commands.spawn((
            PbrBundle {
                mesh: assets.mesh.clone(),
                material: mat,
                transform: *transform,
                ..default()
            },
            TrailDot,
        ));
    }
}

// ── Other marble systems ──────────────────────────────────────────────────────

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
