use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::*;
use egui_plot::{Legend, Line, Plot, PlotPoints};

const DROP_GHOST_COLOR: Color  = Color::srgba(0.95, 0.35, 0.15, 0.75);
const CHUTE_GHOST_COLOR: Color = Color::srgba(0.20, 0.45, 0.90, 0.75);

use crate::resources::constants::MARBLE_RADIUS;
use crate::resources::marble_runs::{RunHistory, VelocitySample};
use crate::systems::marble::{ChuteMarble, FlightTimer, Marble, RunIndex};

pub fn record_chute_marble_system(
    mut all_runs: ResMut<RunHistory>,
    marbles: Query<(&Velocity, &FlightTimer, &RunIndex), (With<Marble>, With<ChuteMarble>)>,
) {
    for (vel, timer, run_idx) in &marbles {
        let sample = VelocitySample {
            t: timer.0,
            vy: vel.linvel.y,
            vz: vel.linvel.z,
            speed: vel.linvel.length(),
            spin: vel.angvel.length() * MARBLE_RADIUS,
        };
        if let Some(run) = all_runs.get_run_mut(run_idx.0) {
            run.samples.push(sample);
        }
    }
}

pub fn draw_marble_ghosts_system(mut gizmos: Gizmos, all_runs: Res<RunHistory>) {
    for run in &all_runs.runs {
        if run.show_ghost {
            if run.drop_path.len() >= 2 {
                gizmos.linestrip(run.drop_path.iter().copied(), DROP_GHOST_COLOR);
            }
            if run.chute_path.len() >= 2 {
                gizmos.linestrip(run.chute_path.iter().copied(), CHUTE_GHOST_COLOR);
            }
        }
    }
}

pub fn marble_graph_ui(mut contexts: EguiContexts, mut all_runs: ResMut<RunHistory>) {
    let ctx = contexts.ctx_mut();

    for run in &mut all_runs.runs {
        if !run.graph_open { continue; }

        let title = format!("Run {} — Velocity", run.index + 1);
        let mut open = true;

        egui::Window::new(&title)
            .id(egui::Id::new(("graph", run.index)))
            .default_size([380.0, 240.0])
            .open(&mut open)
            .show(ctx, |ui| {
                if run.samples.is_empty() {
                    ui.label("No data — marble still in flight or not yet spawned");
                    return;
                }

                let pts = |f: fn(&VelocitySample) -> f64| -> PlotPoints {
                    PlotPoints::from_iter(run.samples.iter().map(|s| [s.t as f64, f(s)]))
                };

                Plot::new(format!("run_{}_vel", run.index))
                    .legend(Legend::default())
                    .height(ui.available_height())
                    .x_axis_label("t (s)")
                    .y_axis_label("m/s")
                    .show(ui, |p| {
                        p.line(Line::new(pts(|s| s.vy as f64)).name("vy"));
                        p.line(Line::new(pts(|s| s.vz as f64)).name("vz"));
                        p.line(Line::new(pts(|s| s.speed as f64)).name("speed"));
                        p.line(Line::new(pts(|s| s.spin as f64)).name("spin"));
                    });
            });

        if !open { run.graph_open = false; }
    }
}
