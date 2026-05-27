use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use avian3d::prelude::{AngularVelocity, LinearVelocity};

use crate::components::snare::{PivotArm, SnareDrum};
use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::resources::marble_runs::{HitRecord, Run, RunHistory};
use crate::resources::programming_wheel_params::{channel_name, WHEEL_CH_VIB_FIRST};
use crate::systems::marble::{Marble, SpawnChannel};

pub fn hud_panel_ui(
    mut contexts: EguiContexts,
    marbles: Query<(&LinearVelocity, &AngularVelocity, &SpawnChannel), With<Marble>>,
    snare: Query<&GlobalTransform, With<SnareDrum>>,
    arm: Query<&Transform, With<PivotArm>>,
    chute_params: Res<ChuteParams>,
    mut all_runs: ResMut<RunHistory>,
) {
    let ctx = contexts.ctx_mut().unwrap();

    // ── Stats panel ───────────────────────────────────────────────────────────
    egui::Window::new("Stats")
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(90.0, -8.0))
        .default_size([340.0, 460.0])
        .resizable(true)
        .title_bar(false)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new(egui::RichText::new("Stats").strong())
                .id_salt("stats_header")
                .default_open(true)
                .show(ui, |ui| {
                    // System info
                    egui::Grid::new("system_grid")
                        .num_columns(2)
                        .spacing([8.0, 2.0])
                        .show(ui, |ui| {
                            if let Ok(arm_tf) = arm.single() {
                                let deg = arm_tf.rotation.to_euler(EulerRot::XYZ).0.to_degrees();
                                ui.label("Pivot");
                                ui.monospace(format!("{:+6.2}°  (− = snare down)", deg));
                                ui.end_row();

                                // Gravitational torque about the pivot X-axis from the two
                                // explicit masses. Positive = CW-side torque (snare rises).
                                let d_cw    = CW_LOCAL_Z    - PIVOT_LOCAL_Z; // +CW_DISTANCE
                                let d_snare = SNARE_LOCAL_Z - PIVOT_LOCAL_Z; // -PIVOT_FROM_SNARE
                                let torque = 9.81 * deg.to_radians().cos()
                                    * (CW_MASS * d_cw + SNARE_MASS * d_snare);
                                ui.label("Torque");
                                ui.monospace(format!("{:+.3} N·m", torque));
                                ui.end_row();

                                let asm_mass = SNARE_MASS + CW_MASS;
                                ui.label("Weight");
                                ui.monospace(format!("{:.2} kg  ({:.1} N)", asm_mass, asm_mass * 9.81));
                                ui.end_row();

                                let rad = deg.to_radians();
                                let tip_h = SNARE_HALF_HEIGHT * rad.cos()
                                    + (PIVOT_FROM_SNARE + SNARE_RADIUS) * rad.sin();
                                ui.label("Tip H");
                                ui.monospace(format!("{:+6.1} cm", tip_h * 100.0));
                                ui.end_row();
                            }
                            let geo = chute_params.geometry();
                            let dz = geo.slope_start[0] - chute_params.exit_pos[0];
                            let dy = geo.slope_start[1] - chute_params.exit_pos[1];
                            let length = (dz * dz + dy * dy).sqrt();
                            let angle = dy.atan2(dz).to_degrees();
                            ui.label("Ramp");
                            ui.monospace(format!("{:.3} m  {:.1}°", length, angle));
                            ui.end_row();
                        });

                    ui.horizontal(|ui| {
                        let lbl = if all_runs.snare_tip_graph_open { "Hide Tip Graph" } else { "Tip Graph" };
                        if ui.small_button(lbl).clicked() {
                            all_runs.snare_tip_graph_open = !all_runs.snare_tip_graph_open;
                        }
                    });

                    ui.separator();

                    // Live marbles
                    let snare_normal = snare
                        .single()
                        .map(|gt| gt.compute_transform().rotation * Vec3::Y)
                        .unwrap_or(Vec3::Y);

                    let mut live: Vec<(usize, Vec3, Vec3)> = marbles
                        .iter()
                        .map(|(lin_vel, ang_vel, spawn_ch)| (spawn_ch.0, lin_vel.0, ang_vel.0))
                        .collect();
                    live.sort_by_key(|(ch, _, _)| *ch);

                    let live_label = if live.is_empty() {
                        egui::RichText::new("Live").strong()
                    } else {
                        egui::RichText::new(format!("Live ({})", live.len())).strong()
                    };
                    egui::CollapsingHeader::new(live_label)
                        .id_salt("live_header")
                        .default_open(false)
                        .show(ui, |ui| {
                            if live.is_empty() {
                                ui.label("No marbles in flight");
                            } else {
                                egui::Grid::new("live_grid")
                                    .num_columns(6)
                                    .spacing([6.0, 2.0])
                                    .show(ui, |ui| {
                                        for label in ["", "spd", "vy", "vh", "AoA", "spin"] {
                                            ui.monospace(label);
                                        }
                                        ui.end_row();
                                        for (ch, v, angvel) in &live {
                                            let label = channel_name(*ch);
                                            let speed = v.length();
                                            let aoa = if speed > 0.01 {
                                                (*v / speed).dot(snare_normal).abs().clamp(0.0, 1.0).asin().to_degrees()
                                            } else {
                                                0.0
                                            };
                                            let spin = angvel.length() * MARBLE_RADIUS;
                                            let vh = Vec2::new(v.x, v.z).length();
                                            ui.monospace(label);
                                            ui.monospace(format!("{speed:5.2}"));
                                            ui.monospace(format!("{:+5.2}", v.y));
                                            ui.monospace(format!("{vh:5.2}"));
                                            ui.monospace(format!("{aoa:4.1}°"));
                                            ui.monospace(format!("{spin:.3}"));
                                            ui.end_row();
                                        }
                                    });
                            }
                        });
                    ui.separator();

                    // Summary
                    let has_runs = !all_runs.runs.is_empty();
                    if has_runs {
                        egui::CollapsingHeader::new("Summary")
                            .id_salt("summary_header")
                            .default_open(false)
                            .show(ui, |ui| render_summary(ui, &all_runs.runs));
                        ui.separator();
                    }

                    // Run history
                    if !has_runs {
                        ui.label("No runs yet");
                        return;
                    }

                    let force_open = all_runs.force_all_open;
                    all_runs.force_all_open = None;

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Runs").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Reset").clicked() {
                                all_runs.runs.clear();
                                all_runs.next_index = 0;
                            }
                            if ui.small_button("Collapse All").clicked() {
                                all_runs.force_all_open = Some(false);
                            }
                            if ui.small_button("Expand All").clicked() {
                                all_runs.force_all_open = Some(true);
                            }
                            let any_ghost = all_runs.runs.iter().any(|r| r.show_ghost);
                            let ghost_label = if any_ghost { "Hide Ghosts" } else { "Show Ghosts" };
                            if ui.small_button(ghost_label).clicked() {
                                let new_val = !any_ghost;
                                for run in &mut all_runs.runs {
                                    run.show_ghost = new_val;
                                }
                            }
                        });
                    });

                    let run_count = all_runs.runs.len();
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                            for i in (0..run_count).rev() {
                                let header = run_entry_label(&all_runs.runs[i]);
                                let run_id = all_runs.runs[i].index;

                                let state_id = ui.make_persistent_id(("run_header", run_id));
                                let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(), state_id, false,
                                );
                                if let Some(open) = force_open {
                                    state.set_open(open);
                                    state.store(ui.ctx());
                                }

                                state
                                    .show_header(ui, |ui| {
                                        ui.checkbox(&mut all_runs.runs[i].show_ghost, "");
                                        ui.label(&header);
                                    })
                                    .body(|ui| {
                                        let run = &all_runs.runs[i];
                                        match run.hit {
                                            None => { ui.label("  — in flight"); }
                                            Some(r) => render_hit_compact(ui, r, run.spawn_channel),
                                        }
                                        ui.add_space(4.0);
                                        ui.horizontal(|ui| {
                                            let graph_label = if all_runs.runs[i].graph_open { "Hide Graph" } else { "Graph" };
                                            if ui.button(graph_label).clicked() {
                                                all_runs.runs[i].graph_open = !all_runs.runs[i].graph_open;
                                            }
                                        });
                                    });
                            }
                        });
                });
        });

    // ── Help panel — anchored bottom-left when collapsed, free when expanded ─
    let mut help_win = egui::Window::new("Help")
        .title_bar(false)
        .resizable(true)
        .default_size([400.0, 520.0]);
    if !all_runs.help_open {
        help_win = help_win.anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(8.0, -8.0));
    }
    let help_resp = help_win.show(&*ctx, |ui| {
        egui::CollapsingHeader::new(egui::RichText::new("Help").strong())
            .default_open(false)
            .show(ui, |ui| render_help_panel(ui))
    });
    all_runs.help_open = help_resp
        .and_then(|ir| ir.inner)
        .map(|cr| cr.body_response.is_some())
        .unwrap_or(false);
}

fn run_entry_label(run: &Run) -> String {
    let name = channel_name(run.spawn_channel);
    match run.hit {
        None => format!("{} {}   — in flight…", name, run.index + 1),
        Some(r) if run.spawn_channel >= WHEEL_CH_VIB_FIRST => {
            format!("{} {}   fly {:.3} s   spd {:.3}   arm {:+.1}°",
                name, run.index + 1, r.flight_s, r.speed, r.arm_deg)
        }
        Some(r) => {
            let radial = (r.hit_local.x * r.hit_local.x + r.hit_local.z * r.hit_local.z).sqrt();
            format!("{} {}   fly {:.3} s   spd {:.3}   arm {:+.1}°   r {:.1} mm",
                name, run.index + 1, r.flight_s, r.speed, r.arm_deg, radial * 1000.0)
        }
    }
}

fn render_hit_compact(ui: &mut egui::Ui, r: HitRecord, spawn_channel: usize) {
    ui.monospace(format!(
        "  fly {:.3} s   spd {:.3}   AoA {:.1}°   KE {:.2} mJ",
        r.flight_s, r.speed, r.aoa, r.ke_mj
    ));
    if spawn_channel >= WHEEL_CH_VIB_FIRST {
        ui.monospace(format!(
            "  vx/vy/vz  {:+.3}/{:+.3}/{:+.3}   spin {:.3}",
            r.vx, r.vy, r.vz, r.spin
        ));
    } else {
        ui.monospace(format!(
            "  vx/vy/vz  {:+.3}/{:+.3}/{:+.3}   spin {:.3}   arm {:+.2}° ω{:+.1}°/s",
            r.vx, r.vy, r.vz, r.spin, r.arm_deg, r.arm_angvel
        ));
        let radial = (r.hit_local.x * r.hit_local.x + r.hit_local.z * r.hit_local.z).sqrt();
        ui.monospace(format!(
            "  hit local  y{:+.1}mm  r{:.1}mm",
            r.hit_local.y * 1000.0, radial * 1000.0
        ));
    }
}

struct Agg { n: usize, mean: f32, std: f32, min: f32, max: f32 }

impl Agg {
    fn from(v: &[f32]) -> Option<Self> {
        let n = v.len();
        if n == 0 { return None; }
        let mean = v.iter().sum::<f32>() / n as f32;
        let std = if n >= 2 {
            (v.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / (n - 1) as f32).sqrt()
        } else {
            0.0
        };
        let min = v.iter().cloned().fold(f32::MAX, f32::min);
        let max = v.iter().cloned().fold(f32::MIN, f32::max);
        Some(Agg { n, mean, std, min, max })
    }

    fn fmt_mean_std(&self, decimals: usize, unit: &str) -> String {
        if self.n < 2 {
            format!("{:.prec$}{}", self.mean, unit, prec = decimals)
        } else {
            format!("{:.prec$} ±{:.prec$}{}", self.mean, self.std, unit, prec = decimals)
        }
    }

    fn fmt_delta_ms(&self) -> String {
        let sign = if self.mean >= 0.0 { "+" } else { "" };
        if self.n < 2 {
            format!("{}{:.1} ms", sign, self.mean)
        } else {
            format!("{}{:.1} ±{:.1} ms   [{:+.1} … {:+.1}]", sign, self.mean, self.std, self.min, self.max)
        }
    }
}

fn render_summary(ui: &mut egui::Ui, runs: &[Run]) {
    use std::collections::BTreeMap;
    use crate::resources::programming_wheel_params::channel_name;

    // Group completed runs by spawn channel, preserving channel order.
    let mut by_channel: BTreeMap<usize, Vec<&Run>> = BTreeMap::new();
    for run in runs.iter().filter(|r| r.hit.is_some()) {
        by_channel.entry(run.spawn_channel).or_default().push(run);
    }

    if by_channel.is_empty() {
        ui.label("No completed runs yet");
        return;
    }

    egui::Grid::new("summary_grid").num_columns(2).spacing([8.0, 2.0]).show(ui, |ui| {
        for (ch, ch_runs) in &by_channel {
            let name = channel_name(*ch);
            let n = ch_runs.len();

            ui.label(egui::RichText::new(format!("── {} ──", name)).strong());
            ui.monospace(format!("n = {}", n));
            ui.end_row();

            let fly:      Vec<f32> = ch_runs.iter().map(|r| r.hit.unwrap().flight_s).collect();
            let delta_ms: Vec<f32> = fly.iter().map(|&f| (f - DROP_REFERENCE_S) * 1000.0).collect();
            let spd:      Vec<f32> = ch_runs.iter().map(|r| r.hit.unwrap().speed).collect();
            let aoa:      Vec<f32> = ch_runs.iter().map(|r| r.hit.unwrap().aoa).collect();
            let ke:       Vec<f32> = ch_runs.iter().map(|r| r.hit.unwrap().ke_mj).collect();

            if let Some(a) = Agg::from(&delta_ms) {
                ui.label(egui::RichText::new("Δt").strong());
                ui.monospace(a.fmt_delta_ms());
                ui.end_row();
            }
            ui.label(egui::RichText::new("fly").strong());
            ui.monospace(Agg::from(&fly).map_or("--".into(), |a| a.fmt_mean_std(3, " s")));
            ui.end_row();
            ui.label(egui::RichText::new("spd").strong());
            ui.monospace(Agg::from(&spd).map_or("--".into(), |a| a.fmt_mean_std(3, " m/s")));
            ui.end_row();
            ui.label(egui::RichText::new("AoA").strong());
            ui.monospace(Agg::from(&aoa).map_or("--".into(), |a| a.fmt_mean_std(1, "°")));
            ui.end_row();
            ui.label(egui::RichText::new("KE").strong());
            ui.monospace(Agg::from(&ke).map_or("--".into(), |a| a.fmt_mean_std(2, " mJ")));
            ui.end_row();
            ui.separator(); ui.separator(); ui.end_row();
        }
    });
}

fn render_help_panel(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("MM3Sim — Marble Machine Trigger Simulator");
        ui.label(
            "Tune a snare drum trigger so two marbles arrive simultaneously. Click to spawn pairs, \
             then adjust the chute until Δt ≈ 0 ms with minimal variance."
        );

        ui.add_space(8.0);

        egui::CollapsingHeader::new(egui::RichText::new("Controls").strong())
            .id_salt("help_controls")
            .default_open(true)
            .show(ui, |ui| {
                let rows: &[(&str, &str)] = &[
                    ("Left Click",          "Spawn marble pair"),
                    ("Right Click + Drag",  "Orbit camera"),
                    ("Middle Click + Drag", "Pan camera"),
                    ("Scroll Wheel",        "Zoom in / out"),
                    ("Drag handle sphere",  "Move Bézier control point"),
                    ("Drag chute body",     "Translate entire curve"),
                ];
                for (key, desc) in rows {
                    ui.label(egui::RichText::new(*key).monospace());
                    ui.label(*desc);
                    ui.end_row();
                }
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("The Marbles").strong())
            .id_salt("help_marbles")
            .default_open(true)
            .show(ui, |ui| {
                use crate::resources::programming_wheel_params::{channel_color_rgb, channel_name, WHEEL_CH_CHUTE, WHEEL_CH_DROP};
                let channels = [
                    (WHEEL_CH_DROP,  "Falls ~1 m with small random lateral jitter. Flight time nearly fixed by gravity."),
                    (WHEEL_CH_CHUTE, "Slides down the chute, lifts off, then flies to snare. Chute shape controls slide duration and liftoff velocity."),
                ];
                for (ch, desc) in channels {
                    let (r, g, b) = channel_color_rgb(ch);
                    ui.label(egui::RichText::new(channel_name(ch)).weak().color(egui::Color32::from_rgb(r, g, b)));
                    ui.label(desc);
                    ui.end_row();
                }
                ui.add_space(2.0);
                ui.label("Both: 20 mm steel marbles, 14 g mass, restitution 0.60, friction 0.18.");
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("Stats Glossary").strong())
            .id_salt("help_stats")
            .default_open(true)
            .show(ui, |ui| {
                let rows: &[(&str, &str)] = &[
                    ("Δt", "Chute flight − 450 ms reference (theoretical 1 m free-fall). Target: 0 ms."),
                    ("fly", "Flight time from spawn to snare impact (s)."),
                    ("spd", "Impact speed magnitude (m/s)."),
                    ("AoA", "Angle of Attack at impact. 0° = perpendicular (ideal), 90° = grazing."),
                    ("KE", "Kinetic energy at impact in mJ: ½mv², marble mass = 14 g."),
                    ("vx/vy/vz", "Velocity components at impact. y is vertical (down = negative)."),
                    ("spin", "Surface speed ω × r (m/s) — spin rate at impact."),
                ];
                for (term, desc) in rows {
                    ui.label(egui::RichText::new(*term).monospace().strong());
                    ui.label(*desc);
                    ui.end_row();
                }
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("Chute Editor").strong())
            .id_salt("help_chute")
            .default_open(true)
            .show(ui, |ui| {
                ui.label("Cubic Bézier curve in Y–Z plane; coordinates relative to snare centre.");
                ui.add_space(4.0);
                let rows: &[(&str, &str)] = &[
                    ("P3 exit", "Bottom exit — where marbles leave the chute."),
                    ("CP2 handle 2", "Second control point — curvature near exit."),
                    ("CP1 handle 1", "First control point — curvature near entry."),
                    ("P0 entry", "Top entry — where marbles spawn."),
                    ("Straight line", "Pure ramp mode: collapses curve handles to P0–P3 line."),
                    ("Show curve handles", "Toggle yellow (CP2) and orange (CP1) spheres. Hidden in straight-line mode."),
                    ("Show endpoint handles", "Toggle green (P0) and red (P3) endpoints."),
                    ("Marble collisions", "Enable marble–marble physics (off by default)."),
                    ("Reset", "Restore factory Bézier defaults."),
                ];
                for (key, desc) in rows {
                    ui.label(egui::RichText::new(*key).strong());
                    ui.label(*desc);
                    ui.end_row();
                }
                ui.add_space(2.0);
                ui.label("Tip: drag handle spheres directly in 3D to reshape the curve.");
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("Pivot Arm").strong())
            .id_salt("help_pivot")
            .default_open(false)
            .show(ui, |ui| {
                ui.label("Counterweighted arm angle controls snare tilt and affects AoA.");
                let rows: &[(&str, &str)] = &[
                    ("θ < 0", "Snare side lower — tilted toward chute."),
                    ("θ = 0", "Arm level; snare head horizontal."),
                    ("θ > 0", "Counterweight side lower — tilted away from chute."),
                ];
                for (k, v) in rows {
                    ui.label(egui::RichText::new(*k).monospace().strong());
                    ui.label(*v);
                    ui.end_row();
                }
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("Summary Statistics").strong())
            .id_salt("help_summary")
            .default_open(false)
            .show(ui, |ui| {
                let rows: &[(&str, &str)] = &[
                    ("n", "Number of completed runs for that channel."),
                    ("mean", "Average across all completed runs."),
                    ("σ std", "Sample standard deviation (÷ n−1), shown as ± after mean."),
                    ("[min … max]", "Range for Δt only."),
                ];
                for (k, v) in rows {
                    ui.label(egui::RichText::new(*k).monospace().strong());
                    ui.label(*v);
                    ui.end_row();
                }
                ui.add_space(2.0);
                ui.label("Goal: minimise |mean Δt| and σ Δt for tight, consistent triggers.");
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("Velocity & Acceleration Graph").strong())
            .id_salt("help_graph")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(
                    "Each run records velocity samples every physics step. Open the graph for any \
                     run via the \"Graph\" button inside that run's entry. Color matches the channel."
                );
                ui.add_space(2.0);
                egui::Grid::new("help_graph_grid").num_columns(2).spacing([12.0, 3.0]).show(ui, |ui| {
                    let rows: &[(&str, &str)] = &[
                        ("speed",    "Total speed magnitude √(vx²+vy²+vz²).  Solid line."),
                        ("vy",       "Vertical velocity (m/s). Negative while falling.  Dashed."),
                        ("vz",       "Forward velocity along chute / arm axis.  Dotted."),
                        ("spin",     "Surface speed from angular velocity: ω × r.  Short dashes."),
                        ("|a|",      "Smoothed acceleration magnitude in the Y-Z plane (m/s²), \
                                      10 ms rolling window. Shows ~9.81 during free flight, \
                                      higher during chute contact, spike at snare impact."),
                    ];
                    for (k, v) in rows {
                        ui.label(egui::RichText::new(*k).monospace().strong());
                        ui.label(*v);
                        ui.end_row();
                    }
                });
                ui.label("Multiple graphs can be open simultaneously. Close via the × button \
                          or \"Graph\" toggle in the run entry.");
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("Vibraphone Mode").strong())
            .id_salt("help_vib")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(
                    "Click the \"Vib\" dropdown in Stats to switch modes. Each run selects one of 37 \
                     bars (F3–E5). Hit location determines which note plays via a physical node system."
                );
            });
    });
}
