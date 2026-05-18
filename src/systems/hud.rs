use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::Velocity;

use crate::components::snare::{PivotArm, SnareDrum};
use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::resources::marble_runs::{HitRecord, Run, RunHistory};
use crate::systems::marble::{ChuteMarble, Marble};
pub fn hud_panel_ui(
    mut contexts: EguiContexts,
    marbles: Query<(&Velocity, Option<&ChuteMarble>), With<Marble>>,
    snare: Query<&GlobalTransform, With<SnareDrum>>,
    arm: Query<&Transform, With<PivotArm>>,
    chute_params: Res<ChuteParams>,
    mut all_runs: ResMut<RunHistory>,
) {
    let ctx = contexts.ctx_mut();

    // ── Stats panel ───────────────────────────────────────────────────────────
    egui::Window::new("Stats")
        .default_pos([10.0, 10.0])
        .default_size([340.0, 460.0])
        .resizable(true)
        .title_bar(false)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new(egui::RichText::new("Stats").strong())
                .id_source("stats_header")
                .default_open(true)
                .show(ui, |ui| {
                    // System info
                    egui::Grid::new("system_grid")
                        .num_columns(2)
                        .spacing([8.0, 2.0])
                        .show(ui, |ui| {
                            if let Ok(arm_tf) = arm.get_single() {
                                let deg = arm_tf.rotation.to_euler(EulerRot::XYZ).0.to_degrees();
                                ui.label("Pivot");
                                ui.monospace(format!("{:+6.2}°  (− = snare down)", deg));
                                ui.end_row();
                            }
                            let dz = chute_params.p0[0] - chute_params.p3[0];
                            let dy = chute_params.p0[1] - chute_params.p3[1];
                            let length = (dz * dz + dy * dy).sqrt();
                            let angle = dy.atan2(dz).to_degrees();
                            ui.label("Ramp");
                            ui.monospace(format!("{:.3} m  {:.1}°", length, angle));
                            ui.end_row();
                        });

                    ui.separator();

                    // Live marbles
                    let snare_normal = snare
                        .get_single()
                        .map(|gt| gt.compute_transform().rotation * Vec3::Y)
                        .unwrap_or(Vec3::Y);

                    let mut live: Vec<(bool, Vec3, Vec3)> = marbles
                        .iter()
                        .map(|(vel, is_chute)| (is_chute.is_some(), vel.linvel, vel.angvel))
                        .collect();
                    live.sort_by_key(|(is_chute, _, _)| *is_chute as u8);

                    if !live.is_empty() {
                        ui.label(egui::RichText::new("Live").strong());
                        egui::Grid::new("live_grid")
                            .num_columns(6)
                            .spacing([6.0, 2.0])
                            .show(ui, |ui| {
                                for label in ["", "spd", "vy", "vh", "AoA", "spin"] {
                                    ui.monospace(label);
                                }
                                ui.end_row();
                                for (is_chute, v, angvel) in &live {
                                    let label = if *is_chute { "Chute" } else { "Drop " };
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
                        ui.separator();
                    }

                    // Summary
                    let has_runs = !all_runs.runs.is_empty();
                    if has_runs {
                        egui::CollapsingHeader::new("Summary")
                            .id_source("summary_header")
                            .default_open(true)
                            .show(ui, |ui| render_summary(ui, &all_runs.runs));
                        ui.separator();
                    }

                    // Run history
                    if !has_runs {
                        ui.label("No runs yet — click to spawn marbles");
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
                        });
                    });

                    let run_count = all_runs.runs.len();
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                            for i in (0..run_count).rev() {
                                let header = run_header_label(&all_runs.runs[i]);
                                egui::CollapsingHeader::new(&header)
                                    .id_source(all_runs.runs[i].index)
                                    .default_open(false)
                                    .open(force_open)
                                    .show(ui, |ui| {
                                        ui.label(egui::RichText::new("Drop").strong());
                                        match all_runs.runs[i].drop {
                                            None => { ui.label("  — in flight"); }
                                            Some(r) => render_drop_compact(ui, r),
                                        }
                                        ui.add_space(3.0);
                                        ui.label(egui::RichText::new("Chute").strong());
                                        match all_runs.runs[i].chute {
                                            None => { ui.label("  — in flight"); }
                                            Some(r) => render_chute_detail(ui, r),
                                        }
                                        ui.add_space(4.0);
                                        ui.horizontal(|ui| {
                                            let graph_label = if all_runs.runs[i].graph_open { "Hide Graph" } else { "Graph" };
                                            if ui.button(graph_label).clicked() {
                                                all_runs.runs[i].graph_open = !all_runs.runs[i].graph_open;
                                            }
                                            let drop_label = if all_runs.runs[i].drop_ghost_open { "Hide Drop" } else { "Drop Ghost" };
                                            if ui.button(drop_label).clicked() {
                                                all_runs.runs[i].drop_ghost_open = !all_runs.runs[i].drop_ghost_open;
                                            }
                                            let chute_label = if all_runs.runs[i].chute_ghost_open { "Hide Chute" } else { "Chute Ghost" };
                                            if ui.button(chute_label).clicked() {
                                                all_runs.runs[i].chute_ghost_open = !all_runs.runs[i].chute_ghost_open;
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
    let help_resp = help_win.show(ctx, |ui| {
        egui::CollapsingHeader::new(egui::RichText::new("Help").strong())
            .default_open(false)
            .show(ui, |ui| render_help_panel(ui))
    });
    all_runs.help_open = help_resp
        .and_then(|ir| ir.inner)
        .map(|cr| cr.body_response.is_some())
        .unwrap_or(false);
}

fn run_header_label(run: &Run) -> String {
    match (run.drop, run.chute) {
        (Some(d), Some(c)) => {
            let ms = (c.flight_s - d.flight_s) * 1000.0;
            let sign = if ms >= 0.0 { "+" } else { "" };
            let exit_str = run.chute_exit.map_or(String::new(), |p3| {
                format!("   P3 y{:+.1} z{:.1}cm", p3[1] * 100.0, p3[0] * 100.0)
            });
            let arc_str = run.chute_exit.map_or(String::new(), |p3| {
                let p3_world = Vec3::new(
                    CHUTE_END_X,
                    p3[1] + CHUTE_ORIGIN_Y,
                    p3[0] + CHUTE_ORIGIN_Z,
                );
                format!("   arc {:.1}cm", p3_world.distance(c.hit_pos) * 100.0)
            });
            format!("Run {}   Δt {}{:.1} ms   spd {:.2}/{:.2}{}{}",
                run.index + 1, sign, ms, d.speed, c.speed, exit_str, arc_str)
        }
        (Some(_), None) => format!("Run {}   drop hit, chute in flight…", run.index + 1),
        (None, Some(_)) => format!("Run {}   chute hit, drop in flight…", run.index + 1),
        (None, None)    => format!("Run {}   in flight…", run.index + 1),
    }
}

fn render_drop_compact(ui: &mut egui::Ui, r: HitRecord) {
    ui.monospace(format!(
        "  fly {:.3} s   spd {:.3}   AoA {:.1}°   KE {:.2} mJ",
        r.flight_s, r.speed, r.aoa, r.ke_mj
    ));
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

fn render_chute_detail(ui: &mut egui::Ui, r: HitRecord) {
    ui.monospace(format!(
        "  fly {:.3} s   spd {:.3}   AoA {:.1}°   KE {:.2} mJ",
        r.flight_s, r.speed, r.aoa, r.ke_mj
    ));
    ui.monospace(format!(
        "  vx/vy/vz  {:+.3}/{:+.3}/{:+.3}   spin {:.3}   arm {:+.2}° ω{:+.1}°/s",
        r.vx, r.vy, r.vz, r.spin, r.arm_deg, r.arm_angvel
    ));
    let radial = (r.hit_local.x * r.hit_local.x + r.hit_local.z * r.hit_local.z).sqrt();
    ui.monospace(format!(
        "  hit local  y{:+.1}mm  r{:.1}mm",
        r.hit_local.y * 1000.0, radial * 1000.0
    ));
    if let Some(slide) = r.slide_s {
        let free_s = r.flight_s - slide;
        if let (Some(end_vy), Some(end_vz)) = (r.slide_end_vy, r.slide_end_vz) {
            let liftoff = (end_vy * end_vy + end_vz * end_vz).sqrt();
            ui.monospace(format!(
                "  slide {:.3} s   free {:.3} s   liftoff vy/vz {:+.3}/{:+.3}  ({:.3} m/s)",
                slide, free_s, end_vy, end_vz, liftoff
            ));
        } else {
            ui.monospace(format!("  slide {:.3} s   free {:.3} s", slide, free_s));
        }
        if let Some(p) = r.slide_end_pos {
            ui.monospace(format!(
                "  liftoff y{:+.1}cm  z{:.1}cm",
                (p.y - CHUTE_ORIGIN_Y) * 100.0,
                (p.z - CHUTE_ORIGIN_Z) * 100.0,
            ));
        }
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
    let complete: Vec<&Run> = runs.iter().filter(|r| r.drop.is_some() && r.chute.is_some()).collect();
    let n = complete.len();
    if n == 0 {
        ui.label("No complete runs yet");
        return;
    }

    let delta_ms: Vec<f32> = complete.iter().map(|r| (r.chute.unwrap().flight_s - r.drop.unwrap().flight_s) * 1000.0).collect();
    let d_fly:  Vec<f32> = complete.iter().map(|r| r.drop.unwrap().flight_s).collect();
    let d_spd:  Vec<f32> = complete.iter().map(|r| r.drop.unwrap().speed).collect();
    let d_aoa:  Vec<f32> = complete.iter().map(|r| r.drop.unwrap().aoa).collect();
    let d_ke:   Vec<f32> = complete.iter().map(|r| r.drop.unwrap().ke_mj).collect();
    let c_fly:  Vec<f32> = complete.iter().map(|r| r.chute.unwrap().flight_s).collect();
    let c_spd:  Vec<f32> = complete.iter().map(|r| r.chute.unwrap().speed).collect();
    let c_aoa:  Vec<f32> = complete.iter().map(|r| r.chute.unwrap().aoa).collect();
    let c_ke:   Vec<f32> = complete.iter().map(|r| r.chute.unwrap().ke_mj).collect();
    let c_slide: Vec<f32> = complete.iter().filter_map(|r| r.chute.unwrap().slide_s).collect();
    let c_lift: Vec<f32> = complete.iter().filter_map(|r| {
        let c = r.chute.unwrap();
        match (c.slide_end_vy, c.slide_end_vz) {
            (Some(vy), Some(vz)) => Some((vy * vy + vz * vz).sqrt()),
            _ => None,
        }
    }).collect();

    egui::Grid::new("summary_grid").num_columns(2).spacing([8.0, 2.0]).show(ui, |ui| {
        ui.label(egui::RichText::new("n").strong());
        ui.monospace(format!("{} complete runs", n));
        ui.end_row();
        ui.separator(); ui.separator(); ui.end_row();

        if let Some(a) = Agg::from(&delta_ms) {
            ui.label(egui::RichText::new("Δt").strong());
            ui.monospace(a.fmt_delta_ms());
            ui.end_row();
        }
        ui.separator(); ui.separator(); ui.end_row();

        ui.label(egui::RichText::new("Drop fly").strong());
        ui.monospace(Agg::from(&d_fly).map_or("--".into(), |a| a.fmt_mean_std(3, " s")));
        ui.end_row();
        ui.label(egui::RichText::new("Drop spd").strong());
        ui.monospace(Agg::from(&d_spd).map_or("--".into(), |a| a.fmt_mean_std(3, " m/s")));
        ui.end_row();
        ui.label(egui::RichText::new("Drop AoA").strong());
        ui.monospace(Agg::from(&d_aoa).map_or("--".into(), |a| a.fmt_mean_std(1, "°")));
        ui.end_row();
        ui.label(egui::RichText::new("Drop KE").strong());
        ui.monospace(Agg::from(&d_ke).map_or("--".into(), |a| a.fmt_mean_std(2, " mJ")));
        ui.end_row();
        ui.separator(); ui.separator(); ui.end_row();

        ui.label(egui::RichText::new("Chute fly").strong());
        ui.monospace(Agg::from(&c_fly).map_or("--".into(), |a| a.fmt_mean_std(3, " s")));
        ui.end_row();
        ui.label(egui::RichText::new("Chute spd").strong());
        ui.monospace(Agg::from(&c_spd).map_or("--".into(), |a| a.fmt_mean_std(3, " m/s")));
        ui.end_row();
        ui.label(egui::RichText::new("Chute AoA").strong());
        ui.monospace(Agg::from(&c_aoa).map_or("--".into(), |a| a.fmt_mean_std(1, "°")));
        ui.end_row();
        ui.label(egui::RichText::new("Chute KE").strong());
        ui.monospace(Agg::from(&c_ke).map_or("--".into(), |a| a.fmt_mean_std(2, " mJ")));
        ui.end_row();

        if !c_slide.is_empty() {
            ui.label(egui::RichText::new("Slide dur").strong());
            ui.monospace(Agg::from(&c_slide).map_or("--".into(), |a| a.fmt_mean_std(3, " s")));
            ui.end_row();
        }
        if !c_lift.is_empty() {
            ui.label(egui::RichText::new("Liftoff spd").strong());
            ui.monospace(Agg::from(&c_lift).map_or("--".into(), |a| a.fmt_mean_std(3, " m/s")));
            ui.end_row();
        }
    });
}

fn render_help_panel(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Overview");
        ui.label(
            "MM3Sim models a marble-machine trigger for a snare drum: a ball rolled down a \
             shaped chute and a ball dropped from above must arrive at the snare head \
             simultaneously. Tune the chute geometry until Δt is as close to zero — and \
             as consistent — as possible."
        );

        ui.add_space(8.0);

        egui::CollapsingHeader::new(egui::RichText::new("Controls").strong())
            .id_source("help_controls")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("help_controls_grid").num_columns(2).spacing([12.0, 3.0]).show(ui, |ui| {
                    let rows: &[(&str, &str)] = &[
                        ("Left Click",              "Spawn a pair of marbles (drop + chute)"),
                        ("Right Click + Drag",      "Orbit camera"),
                        ("Middle Click + Drag",     "Pan camera"),
                        ("Scroll Wheel",            "Zoom in / out (ignored when cursor is over a panel)"),
                        ("Drag handle sphere",      "Move that Bézier control point"),
                        ("Drag chute body",         "Translate the entire chute curve as a unit"),
                    ];
                    for (key, desc) in rows {
                        ui.monospace(*key);
                        ui.label(*desc);
                        ui.end_row();
                    }
                });
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("The Marbles").strong())
            .id_source("help_marbles")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("help_marbles_grid").num_columns(2).spacing([12.0, 3.0]).show(ui, |ui| {
                    ui.label(egui::RichText::new("Drop marble").color(egui::Color32::from_rgb(242, 89, 38)).strong());
                    ui.label("Falls vertically ~1 m above the snare centre with a small random \
                              lateral jitter. Its flight time is nearly fixed by gravity.");
                    ui.end_row();
                    ui.label(egui::RichText::new("Chute marble").color(egui::Color32::from_rgb(51, 115, 230)).strong());
                    ui.label("Starts at rest at the top of the Bézier chute, slides down, \
                              leaves the surface, then flies to the snare. Chute shape \
                              controls both slide duration and liftoff velocity.");
                    ui.end_row();
                });
                ui.add_space(2.0);
                ui.label("Both marbles are steel: 20 mm diameter, 14 g, restitution 0.60, friction 0.18.");
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("Stats Glossary").strong())
            .id_source("help_stats")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("help_stats_grid").num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
                    let rows: &[(&str, &str)] = &[
                        ("Δt",
                         "Flight-time difference: chute − drop, in ms. \
                          Negative = chute hit early, positive = chute hit late. Target: 0 ms."),
                        ("fly",   "Flight time in seconds from spawn to snare contact."),
                        ("spd",   "Speed magnitude at snare impact (m/s)."),
                        ("AoA",
                         "Angle of Attack: angle between the marble's velocity vector and the \
                          snare surface normal at impact. 0° = perfectly perpendicular \
                          (maximum energy transfer). 90° = grazing."),
                        ("KE",
                         "Kinetic energy at impact: ½mv² in millijoules. Marble mass = 14 g."),
                        ("vx / vy / vz",
                         "Velocity components at impact. x = lateral (across snare), \
                          y = vertical (down = negative), z = along the arm."),
                        ("spin",
                         "Surface speed from angular velocity: ω × r (m/s). \
                          Indicates how much the marble is spinning at impact."),
                        ("slide",
                         "Duration (s) the chute marble stayed in contact with the chute \
                          surface before lifting off."),
                        ("liftoff vy / vz",
                         "Velocity components at the moment the chute marble detaches from \
                          the chute. Determines the free-flight trajectory to the snare."),
                        ("liftoff spd",  "Speed magnitude at liftoff: √(vy² + vz²)."),
                        ("vh (live only)", "Horizontal speed in the XZ plane while in flight."),
                    ];
                    for (term, desc) in rows {
                        ui.label(egui::RichText::new(*term).strong().monospace());
                        ui.label(*desc);
                        ui.end_row();
                    }
                });
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("Chute Editor").strong())
            .id_source("help_chute")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(
                    "The chute is a cubic Bézier curve in the Y–Z plane (height × depth). \
                     All coordinates are relative to the snare top-face centre."
                );
                ui.add_space(4.0);
                egui::Grid::new("help_chute_grid").num_columns(2).spacing([12.0, 3.0]).show(ui, |ui| {
                    let rows: &[(&str, &str)] = &[
                        ("P3  exit",
                         "Bottom exit point — where the marble leaves the chute."),
                        ("CP2 handle 2",
                         "Second Bézier control point. Controls curvature near the exit."),
                        ("CP1 handle 1",
                         "First Bézier control point. Controls curvature near the entry."),
                        ("P0  entry",
                         "Top entry point — where the marble is spawned."),
                        ("Straight line",
                         "Collapses CP1/CP2 onto the P0–P3 line (pure ramp). \
                          Curve handle spheres are hidden automatically in this mode."),
                        ("Show curve handles",
                         "Toggle the yellow (CP1) and orange (CP2) control-point spheres \
                          in the 3D viewport. Has no effect in straight-line mode."),
                        ("Show endpoint handles",
                         "Toggle the green (P0 entry) and red (P3 exit) endpoint spheres."),
                        ("Marble collisions",
                         "Enable physical marble–marble interaction (off by default)."),
                        ("Reset",
                         "Restore all Bézier points to their factory defaults."),
                    ];
                    for (key, desc) in rows {
                        ui.label(egui::RichText::new(*key).strong());
                        ui.label(*desc);
                        ui.end_row();
                    }
                });
                ui.add_space(2.0);
                ui.label(
                    "Tip: drag a handle sphere directly in the 3D view to reshape the curve, \
                     or drag anywhere on the chute body to translate the entire curve at once."
                );
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("Pivot Arm").strong())
            .id_source("help_pivot")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(
                    "The snare drum is mounted on a counterweighted pivot arm. The arm angle \
                     (Pivot θ in the Stats panel) determines the tilt of the snare head, \
                     which affects AoA."
                );
                ui.add_space(2.0);
                egui::Grid::new("help_pivot_grid").num_columns(2).spacing([12.0, 3.0]).show(ui, |ui| {
                    let rows: &[(&str, &str)] = &[
                        ("θ < 0", "Snare-side of the arm is lower (snare tilted toward chute)."),
                        ("θ = 0", "Arm level; snare head is horizontal."),
                        ("θ > 0", "Counterweight side is lower; snare tilted away from chute."),
                        ("Stops", "Physical stop posts prevent the arm rotating beyond ±15–17°."),
                    ];
                    for (k, v) in rows {
                        ui.label(egui::RichText::new(*k).monospace().strong());
                        ui.label(*v);
                        ui.end_row();
                    }
                });
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("Summary Statistics").strong())
            .id_source("help_summary")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("help_summary_grid").num_columns(2).spacing([12.0, 3.0]).show(ui, |ui| {
                    let rows: &[(&str, &str)] = &[
                        ("n",       "Number of complete runs (both marbles reached the snare)."),
                        ("mean",    "Arithmetic average over all complete runs."),
                        ("σ (std)", "Sample standard deviation (divides by n−1). \
                                     Shown as ± after the mean."),
                        ("[a … b]", "Min and max observed values. Only shown for Δt."),
                        ("Reset",   "Clears all run history and resets the run counter to 1."),
                    ];
                    for (k, v) in rows {
                        ui.label(egui::RichText::new(*k).strong().monospace());
                        ui.label(*v);
                        ui.end_row();
                    }
                });
                ui.add_space(2.0);
                ui.label("Goal: minimise |mean Δt| (timing accuracy) and σ Δt (consistency).");
            });

        ui.add_space(4.0);

        egui::CollapsingHeader::new(egui::RichText::new("Velocity Graph").strong())
            .id_source("help_graph")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(
                    "Each run records the chute marble's velocity every frame. Open the graph \
                     for any run via the \"Show Graph\" button inside that run's entry."
                );
                ui.add_space(2.0);
                egui::Grid::new("help_graph_grid").num_columns(2).spacing([12.0, 3.0]).show(ui, |ui| {
                    let rows: &[(&str, &str)] = &[
                        ("vy",    "Vertical velocity (m/s). Negative while falling."),
                        ("vz",    "Forward velocity along the chute / arm axis."),
                        ("speed", "Total speed magnitude √(vx²+vy²+vz²)."),
                        ("spin",  "Surface speed from angular velocity: ω × r."),
                    ];
                    for (k, v) in rows {
                        ui.label(egui::RichText::new(*k).strong().monospace());
                        ui.label(*v);
                        ui.end_row();
                    }
                });
                ui.label("Multiple graphs can be open simultaneously. Close via the × button \
                          or \"Hide Graph\" in the run entry.");
            });
    });
}
