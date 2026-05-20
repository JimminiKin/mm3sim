use avian3d::prelude::*;
use bevy::prelude::*;

use crate::components::instrument::Instrument;
use crate::components::snare::PivotArm;
use crate::resources::constants::MARBLE_RADIUS;
use crate::resources::marble_runs::{HitRecord, RunHistory};
use crate::systems::marble::{ChuteMarble, FlightTimer, Marble, PrevVelocity, RunIndex, SlideRecord};

pub const CH_SNARE: usize = 1;
pub const CH_VIB_FIRST: usize = 2;

#[derive(Clone)]
pub struct HitData {
    pub channel: usize,
    pub marble_entity: Entity,
    pub instrument_entity: Entity,
    pub speed: f32,
}

/// Inter-system bus: populated by detect_instrument_hits, consumed by record_ and play_.
#[derive(Resource, Default)]
pub struct InstrumentHits(pub Vec<HitData>);

/// Detects marble-instrument collisions. Clears the previous frame's hits first.
/// No per-instrument marker filter — any Marble hitting any Instrument triggers this.
pub fn detect_instrument_hits(
    mut collision_events: MessageReader<CollisionStart>,
    marbles: Query<&LinearVelocity, With<Marble>>,
    instruments: Query<&Instrument>,
    mut hits: ResMut<InstrumentHits>,
) {
    hits.0.clear();
    for event in collision_events.read() {
        let (e1, e2) = (event.collider1, event.collider2);
        let (marble_entity, instrument_entity) =
            if marbles.contains(e1) && instruments.contains(e2) {
                (e1, e2)
            } else if marbles.contains(e2) && instruments.contains(e1) {
                (e2, e1)
            } else {
                continue;
            };
        let speed = marbles.get(marble_entity).map(|v| v.0.length()).unwrap_or(0.0);
        let Ok(instr) = instruments.get(instrument_entity) else { continue };
        hits.0.push(HitData {
            channel: instr.channel,
            marble_entity,
            instrument_entity,
            speed,
        });
    }
}

/// Records hit statistics into RunHistory for every entry in InstrumentHits.
pub fn record_instrument_hits(
    hits: Res<InstrumentHits>,
    marbles: Query<
        (
            &Transform,
            &PrevVelocity,
            &FlightTimer,
            &RunIndex,
            Option<&ChuteMarble>,
            Option<&SlideRecord>,
        ),
        With<Marble>,
    >,
    instruments: Query<(&GlobalTransform, &Instrument)>,
    arm_q: Query<&AngularVelocity, With<PivotArm>>,
    mut all_runs: ResMut<RunHistory>,
) {
    for hit in &hits.0 {
        let Ok((tf, prev_vel, flight_timer, run_idx, is_chute, slide)) =
            marbles.get(hit.marble_entity)
        else {
            continue;
        };
        let Ok((instr_gt, instr)) = instruments.get(hit.instrument_entity) else {
            continue;
        };

        let instr_rot = instr_gt.compute_transform().rotation;
        let instr_normal = instr_rot * Vec3::Y;
        let arm_deg = instr_rot.to_euler(EulerRot::XYZ).0.to_degrees();
        let instr_center = instr_gt.translation();
        let hit_local = instr_rot.inverse() * (tf.translation - instr_center);

        let mut record = HitRecord::new(
            prev_vel.linvel,
            prev_vel.angvel,
            instr_normal,
            flight_timer.0,
            MARBLE_RADIUS,
        );
        record.hit_pos = tf.translation;
        record.hit_local = hit_local;
        record.arm_deg = arm_deg;

        let Some(run) = all_runs.get_run_mut(run_idx.0) else { continue };

        if instr.channel == CH_SNARE {
            let arm_angvel = arm_q.single().map(|v| v.0.x.to_degrees()).unwrap_or(0.0);
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
                run.chute.get_or_insert(record);
            } else {
                run.drop.get_or_insert(record);
            }
        } else if instr.channel >= CH_VIB_FIRST {
            let bar_idx = (instr.channel - CH_VIB_FIRST) as u32;
            run.vib.get_or_insert(record);
            run.vib_bar_idx.get_or_insert(bar_idx);
        }
    }
}
