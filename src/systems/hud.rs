use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::snare::{SnareDrum, PivotArm};
use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::systems::marble::{ChuteMarble, Marble, SlideData, SpawnTime};

#[derive(Component)]
pub(crate) struct HudText;

#[derive(Clone, Copy, Default)]
struct HitRecord {
    vx: f32,
    vy: f32,
    speed: f32,
    aoa: f32,
    flight_s: f32,
    slide_s: Option<f32>,
    slide_end_vy: Option<f32>,
    slide_end_vz: Option<f32>,
}

#[derive(Resource, Default)]
pub struct LastSnareHit {
    drop: Option<HitRecord>,
    chute: Option<HitRecord>,
}

pub fn setup_hud(mut commands: Commands) {
    commands.insert_resource(LastSnareHit::default());
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 13.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            ..default()
        }),
        HudText,
    ));
}

/// Every frame: for each ChuteMarble not yet lifted off, check min distance to Bézier centerline.
/// Records liftoff time and velocity on the marble's own SlideData component.
pub fn track_slide_end_system(
    time: Res<Time>,
    chute_params: Res<ChuteParams>,
    mut marbles: Query<(&Transform, &Velocity, &mut SlideData), (With<Marble>, With<ChuteMarble>)>,
) {
    let pts = chute_params.effective_pts();

    for (tf, vel, mut slide_data) in &mut marbles {
        if slide_data.end_time.is_some() {
            continue;
        }

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

        let contact_dist = CHUTE_THICKNESS * 0.5 + MARBLE_RADIUS;
        if min_dist > contact_dist + MARBLE_RADIUS {
            slide_data.end_time = Some(time.elapsed_seconds());
            slide_data.end_vel = Some(vel.linvel);
        }
    }
}

pub fn record_snare_aoa_system(
    mut events: EventReader<CollisionEvent>,
    time: Res<Time>,
    marbles: Query<(&Velocity, &SpawnTime, Option<&ChuteMarble>, Option<&SlideData>), With<Marble>>,
    snares: Query<&GlobalTransform, With<SnareDrum>>,
    mut last_hit: ResMut<LastSnareHit>,
) {
    for event in events.read() {
        let CollisionEvent::Started(e1, e2, _) = event else { continue };

        let (marble_entity, snare_entity) =
            if marbles.contains(*e1) && snares.contains(*e2) {
                (*e1, *e2)
            } else if marbles.contains(*e2) && snares.contains(*e1) {
                (*e2, *e1)
            } else {
                continue;
            };

        let Ok(snare_gt) = snares.get(snare_entity) else { continue };
        let snare_normal = snare_gt.compute_transform().rotation * Vec3::Y;

        let Ok((vel, spawn_time, is_chute, slide_data)) = marbles.get(marble_entity) else { continue };
        let v = vel.linvel;
        let speed = v.length();
        let aoa = if speed > 0.01 {
            (v / speed).dot(snare_normal).abs().clamp(0.0, 1.0).asin().to_degrees()
        } else {
            0.0
        };

        let flight_s = time.elapsed_seconds() - spawn_time.0;
        let (slide_s, slide_end_vy, slide_end_vz) = if is_chute.is_some() {
            if let Some(sd) = slide_data {
                let s = sd.end_time.map(|end_s| end_s - spawn_time.0);
                let (vy, vz) = sd.end_vel
                    .map(|lv| (Some(lv.y), Some(lv.z)))
                    .unwrap_or((None, None));
                (s, vy, vz)
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        let record = HitRecord { vx: v.x, vy: v.y, speed, aoa, flight_s, slide_s, slide_end_vy, slide_end_vz };
        if is_chute.is_some() {
            last_hit.chute = Some(record);
        } else {
            last_hit.drop = Some(record);
        }
    }
}

pub fn update_hud_system(
    marbles: Query<(&Velocity, Option<&ChuteMarble>), With<Marble>>,
    snare: Query<&GlobalTransform, With<SnareDrum>>,
    arm: Query<&Transform, With<PivotArm>>,
    chute_params: Res<crate::resources::chute_params::ChuteParams>,
    last_hit: Res<LastSnareHit>,
    mut hud: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = hud.get_single_mut() else { return };

    let snare_normal = snare
        .get_single()
        .map(|gt| gt.compute_transform().rotation * Vec3::Y)
        .unwrap_or(Vec3::Y);

    let mut live: Vec<(bool, Vec3)> = marbles
        .iter()
        .map(|(vel, is_chute)| (is_chute.is_some(), vel.linvel))
        .collect();
    live.sort_by_key(|(is_chute, _)| *is_chute as u8);

    let mut out = String::new();

    if let Ok(arm_tf) = arm.get_single() {
        let deg = arm_tf.rotation.to_euler(EulerRot::XYZ).0.to_degrees();
        out.push_str(&format!("Pivot  {deg:+6.2}°  (− = snare down)\n"));
    }

    {
        let dz = chute_params.p0[0] - chute_params.p3[0];
        let dy = chute_params.p0[1] - chute_params.p3[1];
        let length = (dz * dz + dy * dy).sqrt();
        let angle = dy.atan2(dz).to_degrees();
        out.push_str(&format!("Ramp   {length:.3} m  {angle:.1}°\n"));
    }

    if live.is_empty() {
        out.push_str("No marbles\n");
    } else {
        for (is_chute, v) in &live {
            let label = if *is_chute { "Chute" } else { "Drop " };
            let speed = v.length();
            let v_vert = v.y.abs();
            let v_horiz = Vec2::new(v.x, v.z).length();
            let aoa = if speed > 0.01 {
                (*v / speed).dot(snare_normal).abs().clamp(0.0, 1.0).asin().to_degrees()
            } else {
                0.0
            };
            out.push_str(&format!(
                "[{label}]  spd {speed:5.2} m/s  vy {v_vert:5.2}  vh {v_horiz:5.2}  AoA {aoa:4.1}°\n"
            ));
        }
    }

    out.push_str("─────────────────── last hit ───────────────────\n");

    for (label, record) in [("Drop ", last_hit.drop), ("Chute", last_hit.chute)] {
        match record {
            None => out.push_str(&format!("[{label}]  --\n")),
            Some(r) => {
                out.push_str(&format!(
                    "[{label}]  fly {t:.3}s  vx {vx:+6.2}  vy {vy:+6.2}  spd {spd:5.2}  AoA {aoa:4.1}°",
                    t = r.flight_s, vx = r.vx, vy = r.vy, spd = r.speed, aoa = r.aoa,
                ));
                if let Some(slide) = r.slide_s {
                    out.push_str(&format!("  slide {slide:.3}s"));
                    if let (Some(vy), Some(vz)) = (r.slide_end_vy, r.slide_end_vz) {
                        out.push_str(&format!("  vy {vy:+5.2}  vz {vz:+5.2}"));
                    }
                }
                out.push('\n');
            }
        }
    }

    match (last_hit.drop, last_hit.chute) {
        (Some(d), Some(c)) => {
            let delta_ms = (c.flight_s - d.flight_s) * 1000.0;
            let (adverb, abs_ms) = if delta_ms >= 0.0 {
                ("late", delta_ms)
            } else {
                ("early", -delta_ms)
            };
            out.push_str(&format!("Chute marble is {abs_ms:.1} ms {adverb}\n"));
        }
        _ => out.push_str("Δt = --\n"),
    }

    text.sections[0].value = out.trim_end().to_string();
}
