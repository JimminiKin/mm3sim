use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::*;
use egui_plot::{Legend, Line, LineStyle, Plot, PlotPoints};

const DROP_COLOR:  egui::Color32 = egui::Color32::from_rgb(242, 89,  38);
const CHUTE_COLOR: egui::Color32 = egui::Color32::from_rgb(51,  115, 230);

const DROP_GHOST_COLOR:  Color = Color::srgba(0.95, 0.35, 0.15, 0.75);
const CHUTE_GHOST_COLOR: Color = Color::srgba(0.20, 0.45, 0.90, 0.75);

/// Record one sample per millisecond (1 kHz). At 10 kHz physics this skips 9
/// out of every 10 steps, keeping each run's sample count under ~2000 even for
/// long chute slides. The acceleration window below stays 10 ms wide.
const SAMPLE_INTERVAL: f32 = 0.001;

/// Number of consecutive samples that span the smoothing window.
/// At 1 kHz, 10 samples = 10 ms — wide enough to damp single-step contact
/// spikes while still resolving the slide/free-flight phases clearly.
const ACCEL_SMOOTH: usize = 10;

use crate::resources::constants::MARBLE_RADIUS;
use crate::resources::marble_runs::{MarbleSample, RunHistory};
use crate::systems::marble::{ChuteMarble, FlightTimer, Marble, RunIndex};

pub fn record_marble_samples_system(
    mut all_runs: ResMut<RunHistory>,
    marbles: Query<(&Velocity, &FlightTimer, &RunIndex, Option<&ChuteMarble>), With<Marble>>,
) {
    for (vel, timer, run_idx, is_chute) in &marbles {
        let Some(run) = all_runs.get_run_mut(run_idx.0) else { continue };
        let samples = if is_chute.is_some() {
            &mut run.chute_samples
        } else {
            &mut run.drop_samples
        };

        // Throttle: skip if not enough time has elapsed since the last stored sample.
        if let Some(last) = samples.last() {
            if timer.0 - last.t < SAMPLE_INTERVAL { continue; }
        }

        samples.push(MarbleSample {
            t: timer.0,
            vy: vel.linvel.y,
            vz: vel.linvel.z,
            speed: vel.linvel.length(),
            spin: vel.angvel.length() * MARBLE_RADIUS,
        });
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

        let title = format!("Run {} — Velocity & Acceleration", run.index + 1);
        let mut open = true;

        egui::Window::new(&title)
            .id(egui::Id::new(("graph", run.index)))
            .default_size([420.0, 400.0])
            .open(&mut open)
            .show(ctx, |ui| {
                let has_drop  = !run.drop_samples.is_empty();
                let has_chute = !run.chute_samples.is_empty();

                if !has_drop && !has_chute {
                    ui.label("No data — marble still in flight or not yet spawned");
                    return;
                }

                // Diagnostic sample counts
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!(
                        "drop {} pts    chute {} pts",
                        run.drop_samples.len(),
                        run.chute_samples.len(),
                    )).small().weak());
                });
                ui.add_space(2.0);

                let avail_h = ui.available_height();
                let vel_h   = avail_h * 0.55;
                let accel_h = avail_h * 0.45 - 6.0;

                // ── Velocity panel ────────────────────────────────────────────
                let vel_pts = |samples: &[MarbleSample], f: fn(&MarbleSample) -> f64| -> PlotPoints {
                    PlotPoints::from_iter(samples.iter().map(|s| [s.t as f64, f(s)]))
                };

                Plot::new(format!("run_{}_vel", run.index))
                    .legend(Legend::default())
                    .height(vel_h)
                    .x_axis_label("t (s)")
                    .y_axis_label("m/s")
                    .show(ui, |p| {
                        if has_drop {
                            p.line(Line::new(vel_pts(&run.drop_samples,  |s| s.speed as f64))
                                .name("drop speed").color(DROP_COLOR));
                            p.line(Line::new(vel_pts(&run.drop_samples,  |s| s.vy as f64))
                                .name("drop vy").color(DROP_COLOR)
                                .style(LineStyle::Dashed { length: 10.0 }));
                        }
                        if has_chute {
                            p.line(Line::new(vel_pts(&run.chute_samples, |s| s.speed as f64))
                                .name("chute speed").color(CHUTE_COLOR));
                            p.line(Line::new(vel_pts(&run.chute_samples, |s| s.vy as f64))
                                .name("chute vy").color(CHUTE_COLOR)
                                .style(LineStyle::Dashed { length: 10.0 }));
                            p.line(Line::new(vel_pts(&run.chute_samples, |s| s.vz as f64))
                                .name("chute vz").color(CHUTE_COLOR)
                                .style(LineStyle::Dotted { spacing: 6.0 }));
                            p.line(Line::new(vel_pts(&run.chute_samples, |s| s.spin as f64))
                                .name("chute spin").color(CHUTE_COLOR)
                                .style(LineStyle::Dashed { length: 4.0 }));
                        }
                    });

                ui.add_space(6.0);

                // ── Acceleration panel ────────────────────────────────────────
                // Smoothed over ACCEL_SMOOTH samples (10 ms at 1 kHz).
                // Uses only vy/vz since vx ≈ 0 for both marble types.
                let accel_pts = |samples: &[MarbleSample]| -> PlotPoints {
                    if samples.len() <= ACCEL_SMOOTH {
                        return PlotPoints::from_iter([]);
                    }
                    PlotPoints::from_iter(samples.windows(ACCEL_SMOOTH + 1).filter_map(|w| {
                        let dt = w[ACCEL_SMOOTH].t - w[0].t;
                        if dt < 1e-6 { return None; }
                        let dvy = w[ACCEL_SMOOTH].vy - w[0].vy;
                        let dvz = w[ACCEL_SMOOTH].vz - w[0].vz;
                        let a = (dvy * dvy + dvz * dvz).sqrt() / dt;
                        Some([w[ACCEL_SMOOTH].t as f64, a as f64])
                    }))
                };

                Plot::new(format!("run_{}_accel", run.index))
                    .legend(Legend::default())
                    .height(accel_h)
                    .x_axis_label("t (s)")
                    .y_axis_label("m/s²")
                    .show(ui, |p| {
                        if has_drop {
                            p.line(Line::new(accel_pts(&run.drop_samples))
                                .name("drop |a|").color(DROP_COLOR));
                        }
                        if has_chute {
                            p.line(Line::new(accel_pts(&run.chute_samples))
                                .name("chute |a|").color(CHUTE_COLOR));
                        }
                    });
            });

        if !open { run.graph_open = false; }
    }
}
