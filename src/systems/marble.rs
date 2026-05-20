use avian3d::prelude::*;
use bevy::math::primitives::Sphere;
use bevy::prelude::*;
use rand::RngExt;

use crate::components::snare::SnareDrum;
use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::resources::layers::GameLayer;
use crate::resources::marble_collisions::MarbleCollisions;
use crate::resources::marble_runs::{HitRecord, RunHistory};
use crate::resources::vibraphone_params::VibraphoneParams;
use crate::systems::chute_handles::HandleDrag;

fn jittered_spawn(snare_top_y: f32) -> Vec3 {
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

fn marble_layers(collide: bool) -> CollisionLayers {
    if collide {
        CollisionLayers::new(GameLayer::Marble, [GameLayer::Default, GameLayer::Marble])
    } else {
        CollisionLayers::new(GameLayer::Marble, [GameLayer::Default])
    }
}

#[derive(Component)]
pub struct Marble;

#[derive(Component)]
pub struct ChuteMarble;

#[derive(Component)]
pub struct VibMarble;

#[derive(Component)]
pub struct RunIndex(pub usize);

#[derive(Component)]
pub struct PathTimer(pub f32);

#[derive(Component)]
pub struct FlightTimer(pub f32);

#[derive(Component, Default)]
pub struct PrevVelocity {
    pub linvel: Vec3,
    pub angvel: Vec3,
}

#[derive(Component, Default)]
pub struct SlideRecord {
    pub end_time: Option<f32>,
    pub end_vel: Option<Vec3>,
    pub end_pos: Option<Vec3>,
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

// ── Spawn systems ─────────────────────────────────────────────────────────────

pub fn spawn_marble_on_click_system(
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chute_params: Res<ChuteParams>,
    vib_params: Res<VibraphoneParams>,
    marble_col: Res<MarbleCollisions>,
    drag: Res<HandleDrag>,
    snare: Query<&GlobalTransform, With<SnareDrum>>,
    mut contexts: bevy_egui::EguiContexts,
    mut all_runs: ResMut<RunHistory>,
) {
    if contexts.ctx_mut().unwrap().wants_pointer_input() {
        return;
    }
    if drag.active.is_some() {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let snare_top_y = snare
        .single()
        .map(|gt| gt.translation().y + SNARE_HALF_HEIGHT)
        .unwrap_or(CHUTE_ORIGIN_Y);

    let spawn_position = jittered_spawn(snare_top_y);

    let run_idx = all_runs.push_new_run();
    if let Some(run) = all_runs.get_run_mut(run_idx) {
        run.chute_exit = Some(chute_params.exit_pos);
    }
    spawn_marble(
        &mut commands,
        &mut meshes,
        &mut materials,
        spawn_position,
        marble_col.0,
        run_idx,
    );
    spawn_chute_marble(
        &mut commands,
        &mut meshes,
        &mut materials,
        &chute_params,
        marble_col.0,
        run_idx,
    );

    if vib_params.spawn_marble {
        spawn_vib_marble(&mut commands, &mut meshes, &mut materials, &vib_params, marble_col.0, run_idx);
    }
}

pub fn spawn_marble(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    collide: bool,
    run_idx: usize,
) {
    commands.spawn((
        Marble,
        RunIndex(run_idx),
        FlightTimer(0.0),
        PathTimer(0.0),
        PrevVelocity::default(),
        marble_pbr(meshes, materials, position, MARBLE_COLOR),
        marble_physics(collide),
    ));
}

pub fn spawn_chute_marble(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chute_params: &ChuteParams,
    collide: bool,
    run_idx: usize,
) {
    let geo = chute_params.geometry();
    let [slope_z, slope_y] = geo.slope_start;
    let [slope_tz, slope_ty] = geo.slope_tangent;
    let normal = Vec3::new(0.0, -slope_tz, slope_ty).normalize_or_zero();

    let chute_centre = Vec3::new(CHUTE_END_X, slope_y + CHUTE_ORIGIN_Y, slope_z + CHUTE_ORIGIN_Z);
    let position = chute_centre + normal * (CHUTE_THICKNESS * 0.5 + MARBLE_RADIUS - 0.001);

    commands.spawn((
        (
            Marble,
            ChuteMarble,
            RunIndex(run_idx),
            FlightTimer(0.0),
            PathTimer(0.0),
            SlideRecord::default(),
            PrevVelocity::default(),
        ),
        marble_pbr(meshes, materials, position, CHUTE_MARBLE_COLOR),
        marble_physics(collide),
    ));
}

pub fn spawn_vib_marble(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: &VibraphoneParams,
    collide: bool,
    run_idx: usize,
) {
    let bar_count = VIB_BAR_COUNT;
    // Drop bar index counts from the high (positive X) end, matching bar order
    let logical_idx = params.drop_bar_index.min(bar_count - 1);
    let bar_x = params.row_x_center
        + ((bar_count - 1 - logical_idx) as f32 - (bar_count - 1) as f32 * 0.5)
        * params.bar_spacing;
    let spawn_pos = Vec3::new(bar_x, params.row_y + VIB_SPAWN_HEIGHT, params.row_z);
    commands.spawn((
        Marble,
        VibMarble,
        FlightTimer(0.0),
        PathTimer(0.0),
        PrevVelocity::default(),
        RunIndex(run_idx),
        marble_pbr(meshes, materials, spawn_pos, (0.20, 0.80, 0.35)),
        marble_physics(collide),
    ));
}

// ── Per-frame systems ─────────────────────────────────────────────────────────

pub fn record_marble_paths_system(
    mut all_runs: ResMut<RunHistory>,
    time: Res<Time<Fixed>>,
    mut marbles: Query<
        (&Transform, Option<&ChuteMarble>, Option<&VibMarble>, &RunIndex, &mut PathTimer),
        With<Marble>,
    >,
) {
    let dt = time.delta_secs();
    for (tf, is_chute, is_vib, run_idx, mut timer) in &mut marbles {
        timer.0 += dt;
        if timer.0 < GHOST_SAMPLE_INTERVAL {
            continue;
        }
        timer.0 -= GHOST_SAMPLE_INTERVAL;
        if let Some(run) = all_runs.get_run_mut(run_idx.0) {
            let path = if is_vib.is_some() {
                &mut run.vib_path
            } else if is_chute.is_some() {
                &mut run.chute_path
            } else {
                &mut run.drop_path
            };
            path.push(tf.translation);
        }
    }
}

pub fn despawn_fallen_marbles_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Marble>>,
) {
    for (entity, transform) in &query {
        if transform.translation.y < DESPAWN_Y {
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

pub fn track_slide_end_system(
    chute_params: Res<ChuteParams>,
    mut marbles: Query<
        (&Transform, &LinearVelocity, &FlightTimer, &mut SlideRecord),
        (With<Marble>, With<ChuteMarble>),
    >,
) {
    let geo = chute_params.geometry();

    for (tf, vel, timer, mut slide) in &mut marbles {
        if slide.end_time.is_some() {
            continue;
        }

        let marble_yz = (tf.translation.y, tf.translation.z);

        let min_dist = (0u32..=48)
            .map(|i| {
                let t = i as f32 / 48.0;
                let (bz, by) = if t < 1.0 / 3.0 {
                    let s = t * 3.0;
                    let [sz, sy] = geo.slope_start;
                    let [az, ay] = geo.arc_start;
                    (sz + s * (az - sz), sy + s * (ay - sy))
                } else if t < 2.0 / 3.0 {
                    let s = (t - 1.0 / 3.0) * 3.0;
                    let theta = geo.theta_start + s * geo.arc_sweep;
                    (
                        geo.center[0] + chute_params.curve_radius * theta.cos(),
                        geo.center[1] + chute_params.curve_radius * theta.sin(),
                    )
                } else {
                    let s = (t - 2.0 / 3.0) * 3.0;
                    let [es_z, es_y] = geo.exit_start;
                    let [ee_z, ee_y] = chute_params.exit_pos;
                    (es_z + s * (ee_z - es_z), es_y + s * (ee_y - es_y))
                };
                let dy = marble_yz.0 - (by + CHUTE_ORIGIN_Y);
                let dz = marble_yz.1 - (bz + CHUTE_ORIGIN_Z);
                (dy * dy + dz * dz).sqrt()
            })
            .fold(f32::MAX, f32::min);

        if min_dist > CHUTE_THICKNESS * 0.5 + MARBLE_RADIUS * 2.0 {
            slide.end_time = Some(timer.0);
            slide.end_vel = Some(vel.0);
            slide.end_pos = Some(tf.translation);
        }
    }
}

pub fn record_snare_hit_system(
    mut events: MessageReader<CollisionStart>,
    marbles: Query<
        (
            &Transform,
            &PrevVelocity,
            &FlightTimer,
            Option<&ChuteMarble>,
            Option<&SlideRecord>,
            &RunIndex,
        ),
        With<Marble>,
    >,
    snares: Query<&GlobalTransform, With<crate::components::snare::SnareDrum>>,
    arm_q: Query<&AngularVelocity, With<crate::components::snare::PivotArm>>,
    mut all_runs: ResMut<RunHistory>,
) {
    for event in events.read() {
        let (e1, e2) = (event.collider1, event.collider2);

        let (marble_entity, snare_entity) = if marbles.contains(e1) && snares.contains(e2) {
            (e1, e2)
        } else if marbles.contains(e2) && snares.contains(e1) {
            (e2, e1)
        } else {
            continue;
        };

        let Ok(snare_gt) = snares.get(snare_entity) else {
            continue;
        };
        let snare_rot = snare_gt.compute_transform().rotation;
        let snare_normal = snare_rot * Vec3::Y;
        let arm_deg = snare_rot.to_euler(EulerRot::XYZ).0.to_degrees();
        let arm_angvel = arm_q
            .single()
            .map(|v| v.0.x.to_degrees())
            .unwrap_or(0.0);

        let Ok((tf, prev_vel, flight_timer, is_chute, slide, run_idx)) = marbles.get(marble_entity)
        else {
            continue;
        };

        let snare_center = snare_gt.translation();
        let hit_local = snare_rot.inverse() * (tf.translation - snare_center);

        let mut record = HitRecord::new(
            prev_vel.linvel,
            prev_vel.angvel,
            snare_normal,
            flight_timer.0,
            MARBLE_RADIUS,
        );
        record.hit_pos = tf.translation;
        record.hit_local = hit_local;
        record.arm_deg = arm_deg;
        record.arm_angvel = arm_angvel;

        if is_chute.is_some() {
            if let Some(s) = slide {
                record.slide_s = s.end_time;
                if let Some(lv) = s.end_vel {
                    record.slide_end_vy = Some(lv.y);
                    record.slide_end_vz = Some(lv.z);
                }
                record.slide_end_pos = s.end_pos;
            }
        }

        let Some(run) = all_runs.get_run_mut(run_idx.0) else {
            continue;
        };
        if is_chute.is_some() {
            run.chute.get_or_insert(record);
        } else {
            run.drop.get_or_insert(record);
        }
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
    chute_slides: Query<(&RunIndex, &SlideRecord), (With<Marble>, With<ChuteMarble>)>,
) {
    if let Some(waiting_idx) = auto.waiting_for {
        let still_on_chute = chute_slides
            .iter()
            .any(|(ri, slide)| ri.0 == waiting_idx && slide.end_time.is_none());
        if still_on_chute {
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

    let spawn_position = jittered_spawn(snare_top_y);

    let run_idx = all_runs.push_new_run();
    if let Some(run) = all_runs.get_run_mut(run_idx) {
        run.chute_exit = Some(params.exit_pos);
    }
    spawn_marble(
        &mut commands,
        &mut meshes,
        &mut materials,
        spawn_position,
        marble_col.0,
        run_idx,
    );
    spawn_chute_marble(
        &mut commands,
        &mut meshes,
        &mut materials,
        &params,
        marble_col.0,
        run_idx,
    );
    auto.waiting_for = Some(run_idx);
    auto.pending -= 1;
    auto.spawned += 1;
}
