use avian3d::prelude::*;
use bevy::math::primitives::Sphere;
use bevy::prelude::*;
use rand::RngExt;

use crate::components::snare::SnareDrum;
use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::resources::layers::GameLayer;
use crate::resources::marble_collisions::MarbleCollisions;
use crate::resources::marble_runs::RunHistory;
use crate::resources::programming_wheel_params::{WHEEL_CH_CHUTE, WHEEL_CH_VIB_FIRST};
use crate::resources::vibraphone_params::VibraphoneParams;

pub fn jittered_spawn(snare_top_y: f32) -> Vec3 {
    let (x_off, z_off) = if MARBLE_SPAWN_JITTER > 0.0 {
        let mut rng = rand::rng();
        (
            rng.random_range(-MARBLE_SPAWN_JITTER..MARBLE_SPAWN_JITTER),
            rng.random_range(-MARBLE_SPAWN_JITTER..MARBLE_SPAWN_JITTER),
        )
    } else {
        (0.0, 0.0)
    };
    Vec3::new(MARBLE_SPAWN_X + x_off, snare_top_y + SPAWN_HEIGHT, z_off)
}

/// Compute the world-space spawn position for a chute marble (surface of the slope entry).
pub fn chute_spawn_pos(params: &ChuteParams) -> Vec3 {
    let geo = params.geometry();
    let [slope_z, slope_y] = geo.slope_start;
    let [slope_tz, slope_ty] = geo.slope_tangent;
    let normal = Vec3::new(0.0, -slope_tz, slope_ty).normalize_or_zero();
    let chute_centre = Vec3::new(CHUTE_END_X, slope_y + CHUTE_ORIGIN_Y, slope_z + CHUTE_ORIGIN_Z);
    chute_centre + normal * (CHUTE_THICKNESS * 0.5 + MARBLE_RADIUS - 0.001)
}

/// Compute the world-space spawn position for a vibraphone marble above a bar.
pub fn vib_spawn_pos(params: &VibraphoneParams, bar_idx: u32) -> Vec3 {
    let bar_count = VIB_BAR_COUNT;
    let logical_idx = bar_idx.min(bar_count - 1);
    let bar_x = params.row_x_center
        + ((bar_count - 1 - logical_idx) as f32 - (bar_count - 1) as f32 * 0.5)
        * params.bar_spacing;
    Vec3::new(bar_x, params.row_y + VIB_SPAWN_HEIGHT, params.row_z)
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
    let (color, despawn_floor) = if spawn_channel == WHEEL_CH_CHUTE {
        (CHUTE_MARBLE_COLOR, CHUTE_MARBLE_DESPAWN_Y)
    } else if spawn_channel >= WHEEL_CH_VIB_FIRST {
        (VIB_MARBLE_COLOR, DESPAWN_Y)
    } else {
        (MARBLE_COLOR, DESPAWN_Y)
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
        (&Transform, &SpawnChannel, &RunIndex, &mut PathTimer),
        With<Marble>,
    >,
) {
    let dt = time.delta_secs();
    for (tf, spawn_ch, run_idx, mut timer) in &mut marbles {
        timer.0 += dt;
        if timer.0 < GHOST_SAMPLE_INTERVAL {
            continue;
        }
        timer.0 -= GHOST_SAMPLE_INTERVAL;
        if let Some(run) = all_runs.get_run_mut(run_idx.0) {
            let path = match spawn_ch.0 {
                c if c >= WHEEL_CH_VIB_FIRST => &mut run.vib_path,
                WHEEL_CH_CHUTE               => &mut run.chute_path,
                _                            => &mut run.drop_path,
            };
            path.push(tf.translation);
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
    pub waiting_for: Option<usize>,
}

impl Default for AutoSpawn {
    fn default() -> Self {
        Self {
            batch_size: 100,
            step_exit_y_mm: 0.0,
            step_slope_angle_deg: 0.0,
            pending: 0,
            spawned: 0,
            waiting_for: None,
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
    snare: Query<&GlobalTransform, With<SnareDrum>>,
    mut all_runs: ResMut<RunHistory>,
    // Wait until no chute marble with the tracked run index remains in the world.
    chute_marbles: Query<(&RunIndex, &SpawnChannel), With<Marble>>,
) {
    if let Some(waiting_idx) = auto.waiting_for {
        let still_live = chute_marbles
            .iter()
            .any(|(ri, ch)| ri.0 == waiting_idx && ch.0 == WHEEL_CH_CHUTE);
        if still_live {
            return;
        }
        auto.waiting_for = None;
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

    let snare_top_y = snare
        .single()
        .map(|gt| gt.translation().y + SNARE_HALF_HEIGHT)
        .unwrap_or(CHUTE_ORIGIN_Y);

    let run_idx = all_runs.push_new_run();
    if let Some(run) = all_runs.get_run_mut(run_idx) {
        run.chute_exit = Some(params.exit_pos);
    }

    let drop_pos = jittered_spawn(snare_top_y);
    spawn_marble(&mut commands, &mut meshes, &mut materials,
        drop_pos, marble_col.0, run_idx, crate::resources::programming_wheel_params::WHEEL_CH_DROP);

    let chute_pos = chute_spawn_pos(&params);
    spawn_marble(&mut commands, &mut meshes, &mut materials,
        chute_pos, marble_col.0, run_idx, WHEEL_CH_CHUTE);

    auto.waiting_for = Some(run_idx);
    auto.pending -= 1;
    auto.spawned += 1;
}
