//! Marble lifecycle: spawning, physics, flight tracking, and despawn.
//!
//! `spawn_marble()` is the single entry point for creating a marble.
//! All per-marble properties (colour, despawn floor) are derived from the spawn channel.
//!
//! `AutoSpawn` drives the sweep-and-record mode in `chute_editor.rs`.

use avian3d::prelude::*;
use bevy::math::primitives::Sphere;
use bevy::prelude::*;
use rand::RngExt;

use crate::components::instrument::MarbleSpawner;
use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::resources::layers::GameLayer;
use crate::resources::marble_collisions::MarbleCollisions;
use crate::resources::marble_runs::RunHistory;
use crate::resources::programming_wheel_params::{channel_jitter_xz, channel_target, ChannelTarget, WHEEL_CH_DROP};

/// Compute the world-space spawn position for a chute marble (surface of the slope entry).
///
/// The position is computed in snare-local space (profile at X = 0, in the Y-Z plane)
/// then rotated by `angle_rad` around the Y-axis and offset by `snare_offset`.
pub fn chute_spawn_pos(params: &ChuteParams, snare_offset: Vec3, angle_rad: f32) -> Vec3 {
    let geo = params.geometry();
    let [slope_z, slope_y] = geo.slope_start;
    let [slope_tz, slope_ty] = geo.slope_tangent;
    let normal_local = Vec3::new(0.0, -slope_tz, slope_ty).normalize_or_zero();
    let local = Vec3::new(
        CHUTE_END_X,
        slope_y + CHUTE_ORIGIN_Y,
        slope_z + CHUTE_ORIGIN_Z,
    ) + normal_local * (CHUTE_THICKNESS * 0.5 + MARBLE_RADIUS - 0.001);
    Quat::from_rotation_y(angle_rad) * local + snare_offset
}

fn marble_layers(collide: bool) -> CollisionLayers {
    if collide {
        CollisionLayers::new(GameLayer::Marble, [GameLayer::Default, GameLayer::Marble])
    } else {
        CollisionLayers::new(GameLayer::Marble, [GameLayer::Default])
    }
}

// ── Components ────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Marble;

/// Records which programming-wheel channel (WHEEL_CH_*) spawned this marble.
/// Used everywhere the old `ChuteMarble` / `VibMarble` type tags were used for
/// routing stats, paths, and display labels.
#[derive(Component, Clone, Copy)]
pub struct SpawnChannel(pub usize);

#[derive(Component)]
pub struct RunIndex(pub usize);

#[derive(Component)]
pub struct PathTimer(pub f32);

#[derive(Component)]
pub struct FlightTimer(pub f32);

/// When present, overrides the global `DESPAWN_Y` floor for this marble.
#[derive(Component)]
pub struct DespawnFloor(pub f32);

#[derive(Component, Default)]
pub struct PrevVelocity {
    pub linvel: Vec3,
    pub angvel: Vec3,
}

const GHOST_SAMPLE_INTERVAL: f32 = 0.008;

// ── Shared marble helpers ─────────────────────────────────────────────────────

fn marble_pbr(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    color: (f32, f32, f32),
) -> (Mesh3d, MeshMaterial3d<StandardMaterial>, Transform) {
    (
        Mesh3d(meshes.add(Mesh::from(Sphere {
            radius: MARBLE_RADIUS,
        }))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(color.0, color.1, color.2),
            metallic: MARBLE_METALLIC,
            perceptual_roughness: MARBLE_ROUGHNESS,
            ..default()
        })),
        Transform::from_translation(position),
    )
}

fn marble_physics(collide: bool) -> impl Bundle {
    (
        RigidBody::Dynamic,
        Collider::sphere(MARBLE_RADIUS),
        Mass(MARBLE_MASS),
        Restitution::new(STEEL_RESTITUTION),
        Friction::new(STEEL_FRICTION),
        marble_layers(collide),
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
        SweptCcd::default(),
    )
}

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawn a marble.  All marble-type-specific properties (colour, despawn floor,
/// `SlideRecord`) are derived from `spawn_channel` (one of the `WHEEL_CH_*`
/// constants).  Call sites only need to provide the world position.
pub fn spawn_marble(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    collide: bool,
    run_idx: usize,
    spawn_channel: usize,
) {
    let (color, despawn_floor) = match channel_target(spawn_channel) {
        ChannelTarget::GhostSnare        => (CHUTE_MARBLE_COLOR,  CHUTE_MARBLE_DESPAWN_Y),
        ChannelTarget::VibBar { .. }     => (VIB_MARBLE_COLOR,    DESPAWN_Y),
        ChannelTarget::Snare { .. }      => (MARBLE_COLOR,         DESPAWN_Y),
        ChannelTarget::HiHat { .. }
        | ChannelTarget::HiHatPedal     => (HIHAT_MARBLE_COLOR,   DESPAWN_Y),
    };

    commands.spawn((
        Marble,
        SpawnChannel(spawn_channel),
        RunIndex(run_idx),
        FlightTimer(0.0),
        PathTimer(0.0),
        PrevVelocity::default(),
        DespawnFloor(despawn_floor),
        marble_pbr(meshes, materials, position, color),
        marble_physics(collide),
    ));
}

// ── Per-frame systems ─────────────────────────────────────────────────────────

pub fn record_marble_paths_system(
    mut all_runs: ResMut<RunHistory>,
    time: Res<Time<Fixed>>,
    mut marbles: Query<
        (&Transform, &RunIndex, &mut PathTimer),
        With<Marble>,
    >,
) {
    let dt = time.delta_secs();
    for (tf, run_idx, mut timer) in &mut marbles {
        timer.0 += dt;
        if timer.0 < GHOST_SAMPLE_INTERVAL {
            continue;
        }
        timer.0 -= GHOST_SAMPLE_INTERVAL;
        if let Some(run) = all_runs.get_run_mut(run_idx.0) {
            run.path.push(tf.translation);
        }
    }
}

pub fn despawn_fallen_marbles_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &DespawnFloor), With<Marble>>,
) {
    for (entity, transform, floor) in &query {
        if transform.translation.y < floor.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_marble_collisions(
    mut commands: Commands,
    settings: Res<MarbleCollisions>,
    marbles: Query<Entity, With<Marble>>,
) {
    if !settings.is_changed() {
        return;
    }
    let layers = marble_layers(settings.0);
    for entity in &marbles {
        commands.entity(entity).insert(layers);
    }
}

pub fn capture_prev_velocity_system(
    mut marbles: Query<(&LinearVelocity, &AngularVelocity, &mut PrevVelocity), With<Marble>>,
) {
    for (lin_vel, ang_vel, mut prev) in &mut marbles {
        prev.linvel = lin_vel.0;
        prev.angvel = ang_vel.0;
    }
}

pub fn advance_flight_timers_system(
    time: Res<Time<Fixed>>,
    mut marbles: Query<&mut FlightTimer, With<Marble>>,
) {
    let dt = time.delta_secs();
    for mut timer in &mut marbles {
        timer.0 += dt;
    }
}

#[derive(Resource)]
pub struct AutoSpawn {
    pub batch_size: u32,
    pub step_exit_y_mm: f32,
    pub step_slope_angle_deg: f32,
    pub pending: u32,
    pub spawned: u32,
    /// True while waiting for all live chute marbles (GhostSnare channels) to despawn before
    /// firing the next batch step.
    pub waiting: bool,
}

impl Default for AutoSpawn {
    fn default() -> Self {
        Self {
            batch_size: 100,
            step_exit_y_mm: 0.0,
            step_slope_angle_deg: 0.0,
            pending: 0,
            spawned: 0,
            waiting: false,
        }
    }
}

pub fn auto_spawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut auto: ResMut<AutoSpawn>,
    mut params: ResMut<ChuteParams>,
    marble_col: Res<MarbleCollisions>,
    mut all_runs: ResMut<RunHistory>,
    spawners: Query<(&MarbleSpawner, &Transform)>,
    // Wait until all chute marbles from the previous batch have despawned.
    live_marbles: Query<&SpawnChannel, With<Marble>>,
) {
    if auto.waiting {
        let still_live = live_marbles.iter().any(|ch| matches!(channel_target(ch.0), ChannelTarget::GhostSnare));
        if still_live {
            return;
        }
        auto.waiting = false;
    }

    if auto.pending == 0 {
        return;
    }

    if auto.spawned > 0 {
        if auto.step_exit_y_mm != 0.0 {
            params.exit_pos[1] += auto.step_exit_y_mm * 0.001;
        }
        if auto.step_slope_angle_deg != 0.0 {
            params.slope_angle =
                (params.slope_angle + auto.step_slope_angle_deg).clamp(1.0, 85.0);
        }
        if auto.step_exit_y_mm != 0.0 || auto.step_slope_angle_deg != 0.0 {
            params.dirty = true;
        }
    }

    let mut rng = rand::rng();

    // Drop marble — apply per-channel jitter on top of spawner position.
    let drop_run_idx = all_runs.push_new_run(WHEEL_CH_DROP);
    let mut drop_pos = spawners
        .iter()
        .find(|(s, _)| s.channel == WHEEL_CH_DROP)
        .map(|(_, tf)| tf.translation)
        .unwrap_or_default();
    let jitter = channel_jitter_xz(WHEEL_CH_DROP);
    if jitter > 0.0 {
        drop_pos.x += rng.random_range(-jitter..jitter);
        drop_pos.z += rng.random_range(-jitter..jitter);
    }
    spawn_marble(&mut commands, &mut meshes, &mut materials,
        drop_pos, marble_col.0, drop_run_idx, WHEEL_CH_DROP);

    // One chute marble per chute spawner (each has its own GhostSnare channel).
    for (spawner, tf) in spawners.iter().filter(|(s, _)| matches!(channel_target(s.channel), ChannelTarget::GhostSnare)) {
        let ch = spawner.channel;
        let chute_run_idx = all_runs.push_new_run(ch);
        spawn_marble(&mut commands, &mut meshes, &mut materials,
            tf.translation, marble_col.0, chute_run_idx, ch);
    }

    auto.waiting = true;
    auto.pending -= 1;
    auto.spawned += 1;
}
