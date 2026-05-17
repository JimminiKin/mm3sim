use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::*;
use egui_plot::{Legend, Line, Plot, PlotPoints};

use crate::resources::constants::MARBLE_RADIUS;
use crate::resources::marble_runs::{AllMarbleRuns, HistorySample};
use crate::systems::marble::{ChuteMarble, Marble, RunIndex, SpawnTime};

/// Records one sample per frame for every live ChuteMarble, stored in its run's samples.
pub fn record_chute_marble_system(
    mut all_runs: ResMut<AllMarbleRuns>,
    time: Res<Time>,
    marbles: Query<(&Velocity, &SpawnTime, &RunIndex), (With<Marble>, With<ChuteMarble>)>,
) {
    for (vel, spawn_time, run_idx) in &marbles {
        let t = time.elapsed_seconds() - spawn_time.0;
        let sample = HistorySample {
            t,
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

/// Renders one graph window per run that has graph_open == true.
pub fn marble_graph_ui(mut contexts: EguiContexts, mut all_runs: ResMut<AllMarbleRuns>) {
    let ctx = contexts.ctx_mut();

    for run in &mut all_runs.runs {
        if !run.graph_open {
            continue;
        }

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

                let pts = |f: fn(&HistorySample) -> f64| -> PlotPoints {
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

        if !open {
            run.graph_open = false;
        }
    }
}
