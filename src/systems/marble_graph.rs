use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use avian3d::prelude::*;
use egui_plot::{Legend, Line, LineStyle, Plot, PlotPoints, VLine};

/// Record one sample per millisecond (1 kHz). At 10 kHz physics this skips 9
/// out of every 10 steps, keeping each run's sample count under ~2000 even for
/// long chute slides. The acceleration window below stays 10 ms wide.
const SAMPLE_INTERVAL: f32 = 0.001;

/// Number of consecutive samples that span the smoothing window.
/// At 1 kHz, 10 samples = 10 ms — wide enough to damp single-step contact
/// spikes while still resolving the slide/free-flight phases clearly.
const ACCEL_SMOOTH: usize = 10;

use crate::components::snare::PivotArm;
use crate::resources::constants::*;
use crate::resources::marble_params::MarbleParams;
use crate::resources::marble_runs::{MarbleSample, RunHistory};
use crate::resources::programming_wheel_params::channel_color_rgb;
use crate::systems::marble::{FlightTimer, Marble, RunIndex, SpawnChannel};

fn channel_egui_color(ch: usize) -> egui::Color32 {
    let (r, g, b) = channel_color_rgb(ch);
    egui::Color32::from_rgb(r, g, b)
}

fn channel_bevy_color(ch: usize) -> Color {
    let (r, g, b) = channel_color_rgb(ch);
    Color::srgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 0.75)
}

pub fn record_marble_samples_system(
    mut all_runs: ResMut<RunHistory>,
    marble_params: Res<MarbleParams>,
    marbles: Query<(&LinearVelocity, &AngularVelocity, &FlightTimer, &RunIndex, &SpawnChannel), With<Marble>>,
) {
    for (lin_vel, ang_vel, timer, run_idx, _spawn_ch) in &marbles {
        let Some(run) = all_runs.get_run_mut(run_idx.0) else { continue };

        // Throttle: skip if not enough time has elapsed since the last stored sample.
        if let Some(last) = run.samples.last() {
            if timer.0 - last.t < SAMPLE_INTERVAL { continue; }
        }

        run.samples.push(MarbleSample {
            t: timer.0,
            vy: lin_vel.0.y,
            vz: lin_vel.0.z,
            speed: lin_vel.0.length(),
            spin: ang_vel.0.length() * marble_params.radius,
        });
    }
}

pub fn draw_marble_ghosts_system(mut gizmos: Gizmos, all_runs: Res<RunHistory>) {
    for run in &all_runs.runs {
        if run.show_ghost && run.path.len() >= 2 {
            let color = channel_bevy_color(run.spawn_channel);
            gizmos.linestrip(run.path.iter().copied(), color);
        }
    }
}

pub fn marble_graph_ui(mut contexts: EguiContexts, mut all_runs: ResMut<RunHistory>) {
    let ctx = contexts.ctx_mut().expect("primary egui context");

    for run in &mut all_runs.runs {
        if !run.graph_open { continue; }

        let title = format!("Run {} — Velocity & Acceleration", run.index + 1);
        let mut open = true;
        let color = channel_egui_color(run.spawn_channel);

        egui::Window::new(&title)
            .id(egui::Id::new(("graph", run.index)))
            .default_size([420.0, 420.0])
            .resizable(true)
            .open(&mut open)
            .show(ctx, |ui| {
                if run.samples.is_empty() {
                    ui.label("No data — marble still in flight or not yet spawned");
                    return;
                }

                // Diagnostic sample count
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!(
                        "{} pts",
                        run.samples.len(),
                    )).small().weak());
                });
                ui.add_space(2.0);

                // Allocate plot heights proportionally to available window height.
                // Subtract ~30 px for the sample-count label, spacers, and padding.
                let available_h = (ui.available_height() - 30.0).max(180.0);
                let vel_h   = available_h * (200.0 / 350.0);
                let accel_h = available_h * (150.0 / 350.0);

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
                        p.line(Line::new("speed", vel_pts(&run.samples, |s| s.speed as f64))
                            .color(color));
                        p.line(Line::new("vy", vel_pts(&run.samples, |s| s.vy as f64))
                            .color(color)
                            .style(LineStyle::Dashed { length: 10.0 }));
                        p.line(Line::new("vz", vel_pts(&run.samples, |s| s.vz as f64))
                            .color(color)
                            .style(LineStyle::Dotted { spacing: 6.0 }));
                        p.line(Line::new("spin", vel_pts(&run.samples, |s| s.spin as f64))
                            .color(color)
                            .style(LineStyle::Dashed { length: 4.0 }));
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
                        p.line(Line::new("|a|", accel_pts(&run.samples))
                            .color(color));
                    });
            });

        if !open { run.graph_open = false; }
    }
}

/// The world-Y height of the snare drum's top-face rim on the far side from the pivot.
/// In arm-local the point is (0, +SNARE_HALF_HEIGHT, SNARE_LOCAL_Z − SNARE_RADIUS);
/// its height is SNARE_HALF_HEIGHT·cos(θ) + (PIVOT_FROM_SNARE + SNARE_RADIUS)·sin(θ).
fn snare_far_tip_height(deg: f32) -> f32 {
    let rad = deg.to_radians();
    SNARE_HALF_HEIGHT * rad.cos() + (PIVOT_FROM_SNARE + SNARE_RADIUS) * rad.sin()
}

pub fn snare_tip_graph_ui(
    mut contexts: EguiContexts,
    mut all_runs: ResMut<RunHistory>,
    arm: Query<&Transform, With<PivotArm>>,
) {
    if !all_runs.snare_tip_graph_open { return; }

    let ctx = contexts.ctx_mut().expect("primary egui context");
    let mut open = true;

    let current_deg = arm
        .single()
        .map(|tf| tf.rotation.to_euler(EulerRot::XYZ).0.to_degrees())
        .unwrap_or(ARM_SPAWN_DEG);

    let curve = PlotPoints::from_iter((-300i32..=300).map(|i| {
        let deg = i as f32 * 0.1;
        [deg as f64, (snare_far_tip_height(deg) * 100.0) as f64]
    }));

    // Use chute channel color for the tip-height curve (blue)
    let curve_color = {
        let (r, g, b) = crate::resources::programming_wheel_params::channel_color_rgb(
            crate::resources::programming_wheel_params::WHEEL_CH_CHUTE_FIRST,
        );
        egui::Color32::from_rgb(r, g, b)
    };

    egui::Window::new("Snare Tip Height vs Angle")
        .id(egui::Id::new("snare_tip_graph"))
        .default_size([420.0, 280.0])
        .resizable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            let cur_h_cm = snare_far_tip_height(current_deg) * 100.0;

            Plot::new("snare_tip_angle_plot")
                .legend(Legend::default())
                .x_axis_label("arm angle (°)")
                .y_axis_label("height (cm)")
                .show(ui, |p| {
                    p.line(Line::new("tip height", curve).color(curve_color));
                    // Joint limits
                    p.vline(
                        VLine::new("rest", -(SNARE_REST_DEG as f64))
                            .color(egui::Color32::GRAY)
                            .style(LineStyle::Dashed { length: 6.0 }),
                    );
                    p.vline(
                        VLine::new("max tilt", -((SNARE_REST_DEG + MAX_TILT_DEG) as f64))
                            .color(egui::Color32::GRAY)
                            .style(LineStyle::Dashed { length: 6.0 }),
                    );
                    // Current arm angle
                    p.vline(VLine::new("current", current_deg as f64).color(egui::Color32::YELLOW));
                });

            ui.monospace(format!(
                "θ = {:+.2}°   tip h = {:+.1} cm",
                current_deg, cur_h_cm
            ));
        });

    if !open {
        all_runs.snare_tip_graph_open = false;
    }
}
