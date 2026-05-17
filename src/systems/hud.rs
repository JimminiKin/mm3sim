use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::snare::SnareDrum;
use crate::systems::marble::{ChuteMarble, Marble};

#[derive(Component)]
pub(crate) struct HudText;

#[derive(Clone, Copy, Default)]
struct HitRecord {
    vx: f32,
    vy: f32,
    speed: f32,
    aoa: f32,
    time_s: f32,
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

pub fn record_snare_aoa_system(
    mut events: EventReader<CollisionEvent>,
    time: Res<Time>,
    marbles: Query<(&Velocity, Option<&ChuteMarble>), With<Marble>>,
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

        let Ok((vel, is_chute)) = marbles.get(marble_entity) else { continue };
        let v = vel.linvel;
        let speed = v.length();
        let aoa = if speed > 0.01 {
            (v / speed).dot(snare_normal).abs().clamp(0.0, 1.0).asin().to_degrees()
        } else {
            0.0
        };

        let record = HitRecord { vx: v.x, vy: v.y, speed, aoa, time_s: time.elapsed_seconds() };
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
            Some(r) => out.push_str(&format!(
                "[{label}]  t={t:.3}s  vx {vx:+6.2}  vy {vy:+6.2}  spd {spd:5.2}  AoA {aoa:4.1}°\n",
                t = r.time_s, vx = r.vx, vy = r.vy, spd = r.speed, aoa = r.aoa,
            )),
        }
    }

    match (last_hit.drop, last_hit.chute) {
        (Some(d), Some(c)) => {
            let delta_ms = (c.time_s - d.time_s) * 1000.0;
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
