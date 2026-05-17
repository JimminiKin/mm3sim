use bevy::prelude::*;

use crate::resources::constants::MARBLE_MASS;

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
    pub slide_s: Option<f32>,
    pub slide_end_vy: Option<f32>,
    pub slide_end_vz: Option<f32>,
}

impl HitRecord {
    pub fn new(v: Vec3, angvel: Vec3, snare_normal: Vec3, flight_s: f32, marble_radius: f32) -> Self {
        let speed = v.length();
        let aoa = if speed > 0.01 {
            (v / speed).dot(snare_normal).abs().clamp(0.0, 1.0).asin().to_degrees()
        } else {
            0.0
        };
        let spin = angvel.length() * marble_radius;
        let ke_mj = 0.5 * MARBLE_MASS * speed * speed * 1000.0;
        Self { vx: v.x, vy: v.y, vz: v.z, speed, aoa, flight_s, spin, ke_mj, ..default() }
    }
}

#[derive(Clone, Copy)]
pub struct VelocitySample {
    pub t: f32,
    pub vy: f32,
    pub vz: f32,
    pub speed: f32,
    pub spin: f32,
}

pub struct Run {
    pub index: usize,
    pub drop: Option<HitRecord>,
    pub chute: Option<HitRecord>,
    pub samples: Vec<VelocitySample>,
    pub graph_open: bool,
}

#[derive(Resource, Default)]
pub struct RunHistory {
    pub runs: Vec<Run>,
    pub next_index: usize,
    /// One-frame override: forces all run CollapsingHeaders open (true) or closed (false).
    pub force_all_open: Option<bool>,
    pub help_open: bool,
}

impl RunHistory {
    pub fn push_new_run(&mut self) -> usize {
        let idx = self.next_index;
        self.next_index += 1;
        self.runs.push(Run {
            index: idx,
            drop: None,
            chute: None,
            samples: Vec::new(),
            graph_open: false,
        });
        idx
    }

    pub fn get_run_mut(&mut self, idx: usize) -> Option<&mut Run> {
        self.runs.iter_mut().rfind(|r| r.index == idx)
    }
}
