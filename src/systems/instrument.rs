//! Marble–instrument hit detection and recording.
//!
//! Two-stage pipeline (both in `FixedUpdate`, after `PhysicsSystems::Last`):
//! 1. `detect_instrument_hits` — reads `CollisionStart` events, populates `InstrumentHits`.
//! 2. `record_instrument_hits` — drains `InstrumentHits`, writes `HitRecord` into `RunHistory`.
//!
//! Using a shared `InstrumentHits` bus lets `play_instrument_sounds` consume the same
//! hit list without re-querying collisions.

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::components::instrument::Instrument;
use crate::components::snare::PivotArm;
use crate::resources::marble_params::MarbleParams;
use crate::resources::marble_runs::{HitRecord, RunHistory};
use crate::components::carousel::{
    CAROUSEL_HIT_CRASH, CAROUSEL_HIT_COWBELL, CAROUSEL_HIT_TAMB, CAROUSEL_HIT_WOOD,
};
use crate::resources::programming_wheel_params::{
    WHEEL_CH_DROP, WHEEL_CH_VIB_FIRST, WHEEL_CH_HIHAT_FIRST,
    WHEEL_CH_KICK_FIRST, WHEEL_CH_RIDE_FIRST,
};
use crate::systems::marble::{FlightTimer, Marble, PrevVelocity, RunIndex, SpawnChannel};

/// Channel number of the snare drum instrument (matches `WHEEL_CH_DROP`).
pub const CH_SNARE: usize = WHEEL_CH_DROP;
/// First vibraphone bar channel (matches `WHEEL_CH_VIB_FIRST`).
pub const CH_VIB_FIRST: usize = WHEEL_CH_VIB_FIRST;
/// Hi-hat cymbal channel (the physical bottom cymbal entity).
pub const CH_HIHAT: usize = WHEEL_CH_HIHAT_FIRST;
/// Kick drum channel (the single physics cylinder).
pub const CH_KICK: usize = WHEEL_CH_KICK_FIRST;
/// Ride cymbal channel (the single physics cymbal).
pub const CH_RIDE: usize = WHEEL_CH_RIDE_FIRST;
/// Carousel instrument hit channels (one per slot, not spawn channels).
pub const CH_CAROUSEL_CRASH: usize = CAROUSEL_HIT_CRASH;
pub const CH_CAROUSEL_COWBELL: usize = CAROUSEL_HIT_COWBELL;
pub const CH_CAROUSEL_TAMB: usize = CAROUSEL_HIT_TAMB;
pub const CH_CAROUSEL_WOOD: usize = CAROUSEL_HIT_WOOD;

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
            &SpawnChannel,
        ),
        With<Marble>,
    >,
    instruments: Query<(&GlobalTransform, &Instrument)>,
    arm_q: Query<&AngularVelocity, With<PivotArm>>,
    mut all_runs: ResMut<RunHistory>,
    marble_params: Res<MarbleParams>,
) {
    for hit in &hits.0 {
        let Ok((tf, prev_vel, flight_timer, run_idx, _spawn_ch)) =
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
            marble_params.radius,
            marble_params.mass,
        );
        record.hit_pos = tf.translation;
        record.hit_local = hit_local;
        record.arm_deg = arm_deg;

        if instr.channel == CH_SNARE {
            let arm_angvel = arm_q.single().map(|v| v.0.x.to_degrees()).unwrap_or(0.0);
            record.arm_angvel = arm_angvel;
        }

        let Some(run) = all_runs.get_run_mut(run_idx.0) else { continue };
        run.hit.get_or_insert(record);
    }
}

