//! Run history and hit data.
//!
//! `RunHistory` is the single source of truth for all marble impact records.
//! Each marble spawns a `Run`; `record_instrument_hits` fills in `Run::hit` on contact.

use bevy::prelude::*;

#[derive(Clone, Copy, Default)]
pub struct HitRecord {
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub speed: f32,
    pub aoa: f32,
    pub flight_s: f32,
    pub spin: f32,
    pub ke_mj: f32,
    pub hit_pos: Vec3,
    pub hit_local: Vec3, // hit_pos in snare-local frame: y = axial, xz = radial
    pub arm_deg: f32,
    pub arm_angvel: f32, // deg/s around X axis at moment of hit
}

impl HitRecord {
    pub fn new(v: Vec3, angvel: Vec3, snare_normal: Vec3, flight_s: f32, marble_radius: f32, marble_mass: f32) -> Self {
        let speed = v.length();
        let aoa = if speed > 0.01 {
            (v / speed).dot(snare_normal).abs().clamp(0.0, 1.0).asin().to_degrees()
        } else {
            0.0
        };
        let spin = angvel.length() * marble_radius;
        let ke_mj = 0.5 * marble_mass * speed * speed * 1000.0;
        Self { vx: v.x, vy: v.y, vz: v.z, speed, aoa, flight_s, spin, ke_mj, ..default() }
    }
}

#[derive(Clone, Copy, Default)]
pub struct MarbleSample {
    pub t: f32,
    pub vy: f32,
    pub vz: f32,
    pub speed: f32,
    pub spin: f32,
}

pub struct Run {
    pub index: usize,
    /// The `WHEEL_CH_*` channel that spawned this marble.
    /// Set at spawn time so the run can be labelled before any hit is recorded.
    pub spawn_channel: usize,
    pub hit: Option<HitRecord>,
    pub samples: Vec<MarbleSample>,
    pub graph_open: bool,
    pub path: Vec<Vec3>,
    pub show_ghost: bool,
}

#[derive(Resource, Default)]
pub struct RunHistory {
    pub runs: Vec<Run>,
    pub next_index: usize,
    /// One-frame override: forces all run CollapsingHeaders open (true) or closed (false).
    pub force_all_open: Option<bool>,
    pub help_open: bool,
    pub snare_tip_graph_open: bool,
}

impl RunHistory {
    pub fn push_new_run(&mut self, spawn_channel: usize) -> usize {
        let idx = self.next_index;
        self.next_index += 1;
        self.runs.push(Run {
            index: idx,
            spawn_channel,
            hit: None,
            samples: Vec::new(),
            graph_open: false,
            path: Vec::new(),
            show_ghost: false,
        });
        idx
    }

    pub fn get_run_mut(&mut self, idx: usize) -> Option<&mut Run> {
        self.runs.iter_mut().rfind(|r| r.index == idx)
    }
}
