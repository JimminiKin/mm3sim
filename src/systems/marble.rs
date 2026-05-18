use bevy::math::primitives::Sphere;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::components::snare::SnareDrum;
use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::resources::marble_collisions::MarbleCollisions;
use crate::resources::marble_runs::{HitRecord, RunHistory};
use crate::systems::chute_handles::HandleDrag;

// Marbles live in GROUP_1. Snare/chute use the Rapier default (ALL).
// When marble-marble collisions are off, the filter only matches GROUP_2 (snare/chute),
// so marbles pass through each other while still hitting the snare.
fn marble_filter(collide: bool) -> Group {
    if collide { Group::GROUP_1 | Group::GROUP_2 } else { Group::GROUP_2 }
}

#[derive(Component)]
pub struct Marble;

#[derive(Component)]
pub struct ChuteMarble;

#[derive(Component)]
pub struct SpawnTime(pub f32);

#[derive(Component)]
pub struct RunIndex(pub usize);

#[derive(Component)]
pub struct TrailTimer(pub f32);

#[derive(Component, Default)]
pub struct SlideRecord {
    pub end_time: Option<f32>,
    pub end_vel: Option<Vec3>,
    pub end_pos: Option<Vec3>,
}

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
    let mesh = meshes.add(Mesh::from(Sphere { radius: TRAIL_DOT_RADIUS }));
    let drop_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(MARBLE_COLOR.0, MARBLE_COLOR.1, MARBLE_COLOR.2, 0.55),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let chute_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(CHUTE_MARBLE_COLOR.0, CHUTE_MARBLE_COLOR.1, CHUTE_MARBLE_COLOR.2, 0.55),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.insert_resource(MarbleTrailAssets { mesh, drop_mat, chute_mat });
}

#[derive(Component)]
pub struct TrailDot;

pub fn spawn_marble_on_click_system(
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chute_params: Res<ChuteParams>,
    marble_col: Res<MarbleCollisions>,
    drag: Res<HandleDrag>,
    // Use fixed-step time so spawn timestamps are on the same clock as velocity samples.
    time: Res<Time<Fixed>>,
    trail_dots: Query<Entity, With<TrailDot>>,
    snare: Query<&GlobalTransform, With<SnareDrum>>,
    mut contexts: bevy_egui::EguiContexts,
    mut all_runs: ResMut<RunHistory>,
) {
    if contexts.ctx_mut().wants_pointer_input() { return; }
    if drag.active.is_some() { return; }
    if !buttons.just_pressed(MouseButton::Left) { return; }

    for entity in &trail_dots {
        commands.entity(entity).despawn();
    }

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

    let run_idx = all_runs.push_new_run();
    if let Some(run) = all_runs.get_run_mut(run_idx) {
        run.chute_exit = Some(chute_params.p3);
    }
    spawn_marble(&mut commands, &mut meshes, &mut materials, spawn_position, marble_col.0, spawn_time, run_idx);
    spawn_chute_marble(&mut commands, &mut meshes, &mut materials, &chute_params, marble_col.0, spawn_time, run_idx);
}

pub fn spawn_marble(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    marble_marble_collide: bool,
    spawn_time: f32,
    run_idx: usize,
) {
    commands.spawn((
        Marble,
        RunIndex(run_idx),
        SpawnTime(spawn_time),
        TrailTimer(0.0),
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere { radius: MARBLE_RADIUS })),
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
    run_idx: usize,
) {
    let pts = chute_params.effective_pts();
    let p0 = pts[0];
    let cp1 = pts[1];

    let dz = 3.0 * (cp1[0] - p0[0]);
    let dy = 3.0 * (cp1[1] - p0[1]);
    let normal = Vec3::new(0.0, -dz, dy).normalize_or_zero();

    let chute_centre = Vec3::new(CHUTE_END_X, p0[1] + CHUTE_ORIGIN_Y, p0[0] + CHUTE_ORIGIN_Z);

    // Embed slightly into the surface: at spawn velocity=0, Rapier's speculative-contact
    // prediction sees no approaching velocity and skips contact for one frame.
    // A small penetration depth forces immediate contact resolution.
    let position = chute_centre + normal * (CHUTE_THICKNESS * 0.5 + MARBLE_RADIUS - 0.004);

    // Nest marker components to stay under Bevy's 15-element flat Bundle limit.
    commands.spawn((
        (Marble, ChuteMarble, RunIndex(run_idx), SpawnTime(spawn_time), TrailTimer(0.0), SlideRecord::default()),
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere { radius: MARBLE_RADIUS })),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(CHUTE_MARBLE_COLOR.0, CHUTE_MARBLE_COLOR.1, CHUTE_MARBLE_COLOR.2),
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
        if timer.0 < TRAIL_INTERVAL_S { continue; }
        timer.0 -= TRAIL_INTERVAL_S;

        let mat = if is_chute.is_some() { assets.chute_mat.clone() } else { assets.drop_mat.clone() };
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
    if !settings.is_changed() { return; }
    let filter = marble_filter(settings.0);
    for mut groups in &mut marbles {
        groups.filters = filter;
    }
}

/// Detects when a chute marble lifts off the chute surface and records the moment.
pub fn track_slide_end_system(
    time: Res<Time>,
    chute_params: Res<ChuteParams>,
    mut marbles: Query<(&Transform, &Velocity, &mut SlideRecord), (With<Marble>, With<ChuteMarble>)>,
) {
    let pts = chute_params.effective_pts();

    for (tf, vel, mut slide) in &mut marbles {
        if slide.end_time.is_some() { continue; }

        let marble_yz = (tf.translation.y, tf.translation.z);

        let min_dist = (0u32..=32).map(|i| {
            let t = i as f32 / 32.0;
            let u = 1.0 - t;
            let [p0, p1, p2, p3] = pts;
            let bz = u*u*u*p0[0] + 3.0*u*u*t*p1[0] + 3.0*u*t*t*p2[0] + t*t*t*p3[0];
            let by = u*u*u*p0[1] + 3.0*u*u*t*p1[1] + 3.0*u*t*t*p2[1] + t*t*t*p3[1];
            let dy = marble_yz.0 - (by + CHUTE_ORIGIN_Y);
            let dz = marble_yz.1 - (bz + CHUTE_ORIGIN_Z);
            (dy * dy + dz * dz).sqrt()
        }).fold(f32::MAX, f32::min);

        if min_dist > CHUTE_THICKNESS * 0.5 + MARBLE_RADIUS * 2.0 {
            slide.end_time = Some(time.elapsed_seconds());
            slide.end_vel = Some(vel.linvel);
            slide.end_pos = Some(tf.translation);
        }
    }
}

/// On snare collision, computes and stores the impact record for that marble's run.
pub fn record_snare_hit_system(
    mut events: EventReader<CollisionEvent>,
    time: Res<Time>,
    marbles: Query<(&Transform, &Velocity, &SpawnTime, Option<&ChuteMarble>, Option<&SlideRecord>, &RunIndex), With<Marble>>,
    snares: Query<&GlobalTransform, With<SnareDrum>>,
    arm_q: Query<&Velocity, With<crate::components::snare::PivotArm>>,
    mut all_runs: ResMut<RunHistory>,
) {
    for event in events.read() {
        let CollisionEvent::Started(e1, e2, _) = event else { continue };

        let (marble_entity, snare_entity) =
            if marbles.contains(*e1) && snares.contains(*e2) { (*e1, *e2) }
            else if marbles.contains(*e2) && snares.contains(*e1) { (*e2, *e1) }
            else { continue };

        let Ok(snare_gt) = snares.get(snare_entity) else { continue };
        let snare_rot = snare_gt.compute_transform().rotation;
        let snare_normal = snare_rot * Vec3::Y;
        let arm_deg = snare_rot.to_euler(EulerRot::XYZ).0.to_degrees();
        let arm_angvel = arm_q.get_single()
            .map(|v| v.angvel.x.to_degrees())
            .unwrap_or(0.0);

        let Ok((tf, vel, spawn_time, is_chute, slide, run_idx)) = marbles.get(marble_entity) else { continue };

        let snare_center = snare_gt.translation();
        let hit_local = snare_rot.inverse() * (tf.translation - snare_center);

        let flight_s = time.elapsed_seconds() - spawn_time.0;
        let mut record = HitRecord::new(vel.linvel, vel.angvel, snare_normal, flight_s, MARBLE_RADIUS);
        record.hit_pos = tf.translation;
        record.hit_local = hit_local;
        record.arm_deg = arm_deg;
        record.arm_angvel = arm_angvel;

        if is_chute.is_some() {
            if let Some(s) = slide {
                record.slide_s = s.end_time.map(|end_s| end_s - spawn_time.0);
                if let Some(lv) = s.end_vel {
                    record.slide_end_vy = Some(lv.y);
                    record.slide_end_vz = Some(lv.z);
                }
                record.slide_end_pos = s.end_pos;
            }
        }

        let Some(run) = all_runs.get_run_mut(run_idx.0) else { continue };
        if is_chute.is_some() { run.chute = Some(record); } else { run.drop = Some(record); }
    }
}
