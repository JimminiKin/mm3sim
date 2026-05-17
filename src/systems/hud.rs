use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::*;

use crate::components::snare::{PivotArm, SnareDrum};
use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::resources::marble_runs::{AllMarbleRuns, HitRecord, MarbleRun};
use crate::systems::marble::{ChuteMarble, Marble, RunIndex, SlideData, SpawnTime};

// ── Physics systems (unchanged) ───────────────────────────────────────────────

pub fn track_slide_end_system(
    time: Res<Time>,
    chute_params: Res<ChuteParams>,
    mut marbles: Query<(&Transform, &Velocity, &mut SlideData), (With<Marble>, With<ChuteMarble>)>,
) {
    let pts = chute_params.effective_pts();

    for (tf, vel, mut slide_data) in &mut marbles {
        if slide_data.end_time.is_some() {
            continue;
        }

        let marble_yz = (tf.translation.y, tf.translation.z);

        let min_dist = (0u32..=32)
            .map(|i| {
                let t = i as f32 / 32.0;
                let u = 1.0 - t;
                let [p0, p1, p2, p3] = pts;
                let bz = u * u * u * p0[0]
                    + 3.0 * u * u * t * p1[0]
                    + 3.0 * u * t * t * p2[0]
                    + t * t * t * p3[0];
                let by = u * u * u * p0[1]
                    + 3.0 * u * u * t * p1[1]
                    + 3.0 * u * t * t * p2[1]
                    + t * t * t * p3[1];
                let dy = marble_yz.0 - (by + CHUTE_ORIGIN_Y);
                let dz = marble_yz.1 - (bz + CHUTE_ORIGIN_Z);
                (dy * dy + dz * dz).sqrt()
            })
            .fold(f32::MAX, f32::min);

        let contact_dist = CHUTE_THICKNESS * 0.5 + MARBLE_RADIUS;
        if min_dist > contact_dist + MARBLE_RADIUS {
            slide_data.end_time = Some(time.elapsed_seconds());
            slide_data.end_vel = Some(vel.linvel);
        }
    }
}

pub fn record_snare_aoa_system(
    mut events: EventReader<CollisionEvent>,
    time: Res<Time>,
    marbles: Query<
        (
            &Velocity,
            &SpawnTime,
            Option<&ChuteMarble>,
            Option<&SlideData>,
            &RunIndex,
        ),
        With<Marble>,
    >,
    snares: Query<&GlobalTransform, With<SnareDrum>>,
    mut all_runs: ResMut<AllMarbleRuns>,
) {
    for event in events.read() {
        let CollisionEvent::Started(e1, e2, _) = event else {
            continue;
        };

        let (marble_entity, snare_entity) = if marbles.contains(*e1) && snares.contains(*e2) {
            (*e1, *e2)
        } else if marbles.contains(*e2) && snares.contains(*e1) {
            (*e2, *e1)
        } else {
            continue;
        };

        let Ok(snare_gt) = snares.get(snare_entity) else {
            continue;
        };
        let snare_normal = snare_gt.compute_transform().rotation * Vec3::Y;

        let Ok((vel, spawn_time, is_chute, slide_data, run_idx)) = marbles.get(marble_entity)
        else {
            continue;
        };

        let flight_s = time.elapsed_seconds() - spawn_time.0;
        let mut record = HitRecord::new(
            vel.linvel,
            vel.angvel,
            snare_normal,
            flight_s,
            MARBLE_RADIUS,
        );

        if is_chute.is_some() {
            if let Some(sd) = slide_data {
                record.slide_s = sd.end_time.map(|end_s| end_s - spawn_time.0);
                if let Some(lv) = sd.end_vel {
                    record.slide_end_vy = Some(lv.y);
                    record.slide_end_vz = Some(lv.z);
                }
            }
        }

        let Some(run) = all_runs.get_run_mut(run_idx.0) else {
            continue;
        };
        if is_chute.is_some() {
            run.chute = Some(record);
        } else {
            run.drop = Some(record);
        }
    }
}

// ── Main UI system ────────────────────────────────────────────────────────────

pub fn hud_panel_ui(
    mut contexts: EguiContexts,
    marbles: Query<(&Velocity, Option<&ChuteMarble>), With<Marble>>,
    snare: Query<&GlobalTransform, With<SnareDrum>>,
    arm: Query<&Transform, With<PivotArm>>,
    chute_params: Res<ChuteParams>,
    mut all_runs: ResMut<AllMarbleRuns>,
) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("Stats")
        .default_pos([10.0, 10.0])
        .default_size([340.0, 460.0])
        .resizable(true)
        .title_bar(false)
        .show(ctx, |ui| {
            // ── System info ──────────────────────────────────────────────────
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

            // ── Live marbles ─────────────────────────────────────────────────
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
                                (*v / speed)
                                    .dot(snare_normal)
                                    .abs()
                                    .clamp(0.0, 1.0)
                                    .asin()
                                    .to_degrees()
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

            // ── Summary ──────────────────────────────────────────────────────
            let has_runs = !all_runs.runs.is_empty();
            if has_runs {
                egui::CollapsingHeader::new("Summary")
                    .id_source("summary_header")
                    .default_open(true)
                    .show(ui, |ui| render_summary(ui, &all_runs.runs));
                ui.separator();
            }

            // ── Run history ──────────────────────────────────────────────────
            if !has_runs {
                ui.label("No runs yet — click to spawn marbles");
                return;
            }

            // Consume any pending expand/collapse-all override.
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
                        let run = &all_runs.runs[i];
                        let header = run_header_label(run);

                        egui::CollapsingHeader::new(&header)
                            .id_source(run.index)
                            .default_open(false)
                            .open(force_open)
                            .show(ui, |ui| {
                                // Drop — compact two-line format
                                ui.label(egui::RichText::new("Drop").strong());
                                match all_runs.runs[i].drop {
                                    None => {
                                        ui.label("  — in flight");
                                    }
                                    Some(r) => render_drop_compact(ui, r),
                                }

                                ui.add_space(3.0);

                                // Chute — full detail
                                ui.label(egui::RichText::new("Chute").strong());
                                match all_runs.runs[i].chute {
                                    None => {
                                        ui.label("  — in flight");
                                    }
                                    Some(r) => render_chute_detail(ui, r),
                                }

                                ui.add_space(4.0);
                                let btn_label = if all_runs.runs[i].graph_open {
                                    "Hide Graph"
                                } else {
                                    "Show Graph"
                                };
                                if ui.button(btn_label).clicked() {
                                    all_runs.runs[i].graph_open = !all_runs.runs[i].graph_open;
                                }
                            });
                    }
                });
        });

    // ── Help window — always present, bottom-left when collapsed ─────────────
    let help_collapsing_id = egui::Id::new("Help").with("collapsing");

    // On the very first frame, write a collapsed initial state so the window
    // starts as just a title bar rather than fully expanded.
    if egui::collapsing_header::CollapsingState::load(ctx, help_collapsing_id).is_none() {
        egui::collapsing_header::CollapsingState::load_with_default_open(
            ctx,
            help_collapsing_id,
            false,
        )
        .store(ctx);
    }

    let help_is_open = egui::collapsing_header::CollapsingState::load(ctx, help_collapsing_id)
        .map(|s| s.is_open())
        .unwrap_or(false);

    // When collapsed: pin to bottom-left so it stays out of the way.
    // When expanded: float freely so the user can reposition it.
    let mut help_window = egui::Window::new("Help")
        .collapsible(true)
        .resizable(true)
        .default_size([400.0, 520.0]);

    if !help_is_open {
        help_window = help_window.anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(8.0, -8.0));
    }

    help_window.show(ctx, render_help_panel);
}

// ── Per-run header label ──────────────────────────────────────────────────────

fn run_header_label(run: &MarbleRun) -> String {
    match (run.drop, run.chute) {
        (Some(d), Some(c)) => {
            let ms = (c.flight_s - d.flight_s) * 1000.0;
            let sign = if ms >= 0.0 { "+" } else { "" };
            format!(
                "Run {}   Δt {}{:.1} ms   spd drop {:.2} / chute {:.2}",
                run.index + 1,
                sign,
                ms,
                d.speed,
                c.speed
            )
        }
        (Some(_), None) => format!("Run {}   drop hit, chute in flight…", run.index + 1),
        (None, Some(_)) => format!("Run {}   chute hit, drop in flight…", run.index + 1),
        (None, None) => format!("Run {}   in flight…", run.index + 1),
    }
}

// ── Compact drop display (2 lines) ───────────────────────────────────────────

fn render_drop_compact(ui: &mut egui::Ui, r: HitRecord) {
    ui.monospace(format!(
        "  fly {:.3} s   spd {:.3}   AoA {:.1}°   KE {:.2} mJ",
        r.flight_s, r.speed, r.aoa, r.ke_mj
    ));
    ui.monospace(format!(
        "  vx/vy/vz  {:+.3}/{:+.3}/{:+.3}   spin {:.3}",
        r.vx, r.vy, r.vz, r.spin
    ));
}

// ── Detailed chute display (grid) ────────────────────────────────────────────

fn render_chute_detail(ui: &mut egui::Ui, r: HitRecord) {
    ui.monospace(format!(
        "  fly {:.3} s   spd {:.3}   AoA {:.1}°   KE {:.2} mJ",
        r.flight_s, r.speed, r.aoa, r.ke_mj
    ));
    ui.monospace(format!(
        "  vx/vy/vz  {:+.3}/{:+.3}/{:+.3}   spin {:.3}",
        r.vx, r.vy, r.vz, r.spin
    ));

    if let Some(slide) = r.slide_s {
        if let (Some(end_vy), Some(end_vz)) = (r.slide_end_vy, r.slide_end_vz) {
            let liftoff = (end_vy * end_vy + end_vz * end_vz).sqrt();
            ui.monospace(format!(
                "  slide {:.3} s   liftoff vy/vz {:+.3}/{:+.3}  ({:.3} m/s)",
                slide, end_vy, end_vz, liftoff
            ));
        } else {
            ui.monospace(format!("  slide {:.3} s", slide));
        }
    }
}

// ── Summary section ───────────────────────────────────────────────────────────

struct Agg {
    n: usize,
    mean: f32,
    std: f32,
    min: f32,
    max: f32,
}

impl Agg {
    fn from(v: &[f32]) -> Option<Self> {
        let n = v.len();
        if n == 0 {
            return None;
        }
        let mean = v.iter().sum::<f32>() / n as f32;
        let std = if n >= 2 {
            (v.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / (n - 1) as f32).sqrt()
        } else {
            0.0
        };
        let min = v.iter().cloned().fold(f32::MAX, f32::min);
        let max = v.iter().cloned().fold(f32::MIN, f32::max);
        Some(Agg {
            n,
            mean,
            std,
            min,
            max,
        })
    }

    /// Format as "mean ± std" with the given decimal places, followed by a unit.
    fn fmt_mean_std(&self, decimals: usize, unit: &str) -> String {
        if self.n < 2 {
            format!("{:.prec$}{}", self.mean, unit, prec = decimals)
        } else {
            format!(
                "{:.prec$} ±{:.prec$}{}",
                self.mean,
                self.std,
                unit,
                prec = decimals
            )
        }
    }

    /// Format Δt: mean ± std with sign, plus range.
    fn fmt_delta_ms(&self) -> String {
        let sign = if self.mean >= 0.0 { "+" } else { "" };
        if self.n < 2 {
            format!("{}{:.1} ms", sign, self.mean)
        } else {
            format!(
                "{}{:.1} ±{:.1} ms   [{:+.1} … {:+.1}]",
                sign, self.mean, self.std, self.min, self.max
            )
        }
    }
}

fn render_summary(ui: &mut egui::Ui, runs: &[MarbleRun]) {
    let complete: Vec<&MarbleRun> = runs
        .iter()
        .filter(|r| r.drop.is_some() && r.chute.is_some())
        .collect();

    let n = complete.len();
    if n == 0 {
        ui.label("No complete runs yet");
        return;
    }

    let delta_ms: Vec<f32> = complete
        .iter()
        .map(|r| (r.chute.unwrap().flight_s - r.drop.unwrap().flight_s) * 1000.0)
        .collect();

    let d_fly: Vec<f32> = complete.iter().map(|r| r.drop.unwrap().flight_s).collect();
    let d_spd: Vec<f32> = complete.iter().map(|r| r.drop.unwrap().speed).collect();
    let d_aoa: Vec<f32> = complete.iter().map(|r| r.drop.unwrap().aoa).collect();
    let d_ke: Vec<f32> = complete.iter().map(|r| r.drop.unwrap().ke_mj).collect();

    let c_fly: Vec<f32> = complete.iter().map(|r| r.chute.unwrap().flight_s).collect();
    let c_spd: Vec<f32> = complete.iter().map(|r| r.chute.unwrap().speed).collect();
    let c_aoa: Vec<f32> = complete.iter().map(|r| r.chute.unwrap().aoa).collect();
    let c_ke: Vec<f32> = complete.iter().map(|r| r.chute.unwrap().ke_mj).collect();

    let c_slide: Vec<f32> = complete
        .iter()
        .filter_map(|r| r.chute.unwrap().slide_s)
        .collect();
    let c_lift: Vec<f32> = complete
        .iter()
        .filter_map(|r| {
            let c = r.chute.unwrap();
            match (c.slide_end_vy, c.slide_end_vz) {
                (Some(vy), Some(vz)) => Some((vy * vy + vz * vz).sqrt()),
                _ => None,
            }
        })
        .collect();

    egui::Grid::new("summary_grid")
        .num_columns(2)
        .spacing([8.0, 2.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new("n").strong());
            ui.monospace(format!("{} complete runs", n));
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Δt
            if let Some(a) = Agg::from(&delta_ms) {
                ui.label(egui::RichText::new("Δt").strong());
                ui.monospace(a.fmt_delta_ms());
                ui.end_row();
            }

            ui.separator();
            ui.separator();
            ui.end_row();

            // Drop
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

            ui.separator();
            ui.separator();
            ui.end_row();

            // Chute
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

// ── Help panel ────────────────────────────────────────────────────────────────

fn render_help_panel(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // ── Overview ─────────────────────────────────────────────────────────
        ui.heading("Overview");
        ui.label(
            "MM3Sim models a marble-machine trigger for a snare drum: a ball \
             rolled down a shaped chute and a ball dropped from above must \
             arrive at the snare head simultaneously. The goal is to tune the \
             chute geometry until the timing offset Δt is as close to zero \
             (and as consistent) as possible."
        );

        ui.add_space(8.0);

        // ── Controls ─────────────────────────────────────────────────────────
        egui::CollapsingHeader::new(egui::RichText::new("Controls").strong())
            .id_source("help_controls")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("help_controls_grid")
                    .num_columns(2)
                    .spacing([12.0, 3.0])
                    .show(ui, |ui| {
                        let rows: &[(&str, &str)] = &[
                            ("Left Click",             "Spawn a pair of marbles (drop + chute)"),
                            ("Right Click + Drag",     "Orbit camera"),
                            ("Middle Click + Drag",    "Pan camera"),
                            ("Scroll Wheel",           "Zoom in / out"),
                            ("Drag blue/cyan handles", "Reshape the chute curve directly in 3D"),
                        ];
                        for (key, desc) in rows {
                            ui.monospace(*key);
                            ui.label(*desc);
                            ui.end_row();
                        }
                    });
            });

        ui.add_space(4.0);

        // ── The Marbles ───────────────────────────────────────────────────────
        egui::CollapsingHeader::new(egui::RichText::new("The Marbles").strong())
            .id_source("help_marbles")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("help_marbles_grid")
                    .num_columns(2)
                    .spacing([12.0, 3.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Drop marble").color(egui::Color32::from_rgb(242, 89, 38)).strong());
                        ui.label("Falls vertically ~1 m above the snare centre with a small \
                                  random lateral jitter. Its flight time is nearly fixed by gravity.");
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

        // ── Stats Glossary ────────────────────────────────────────────────────
        egui::CollapsingHeader::new(egui::RichText::new("Stats Glossary").strong())
            .id_source("help_stats")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("help_stats_grid")
                    .num_columns(2)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        let rows: &[(&str, &str)] = &[
                            ("Δt",
                             "Flight-time difference: chute − drop, in ms. \
                              Negative = chute hit early, positive = chute hit late. \
                              Target: 0 ms."),
                            ("fly",
                             "Flight time in seconds from spawn to snare contact."),
                            ("spd",
                             "Speed magnitude at snare impact (m/s)."),
                            ("AoA",
                             "Angle of Attack: angle between the marble's velocity vector \
                              and the snare surface normal at impact. 0° = perfectly \
                              perpendicular (maximum energy transfer). 90° = grazing."),
                            ("KE",
                             "Kinetic energy at impact: ½mv² in millijoules. \
                              Marble mass = 14 g."),
                            ("vx / vy / vz",
                             "Velocity components at impact. x = lateral (across snare), \
                              y = vertical (down = negative), z = along the arm."),
                            ("spin",
                             "Surface speed from angular velocity: ω × r (m/s). \
                              Indicates how much the marble is spinning at impact, \
                              which affects post-bounce behaviour on the head."),
                            ("slide",
                             "Duration (s) the chute marble stayed in contact with \
                              the chute surface before lifting off."),
                            ("liftoff vy / vz",
                             "Velocity components at the moment the chute marble \
                              detaches from the chute surface. Determines the free-flight \
                              trajectory to the snare."),
                            ("liftoff spd",
                             "Speed magnitude at liftoff: √(vy² + vz²)."),
                            ("vh (live only)",
                             "Horizontal speed in the XZ plane while the marble is in flight."),
                        ];
                        for (term, desc) in rows {
                            ui.label(egui::RichText::new(*term).strong().monospace());
                            ui.label(*desc);
                            ui.end_row();
                        }
                    });
            });

        ui.add_space(4.0);

        // ── Chute Editor ─────────────────────────────────────────────────────
        egui::CollapsingHeader::new(egui::RichText::new("Chute Editor").strong())
            .id_source("help_chute")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(
                    "The chute is a cubic Bézier curve in the Y–Z plane \
                     (height × depth). All coordinates are relative to the \
                     snare top-face centre."
                );
                ui.add_space(4.0);
                egui::Grid::new("help_chute_grid")
                    .num_columns(2)
                    .spacing([12.0, 3.0])
                    .show(ui, |ui| {
                        let rows: &[(&str, &str)] = &[
                            ("P0  start",    "Top end of the chute — where the marble is spawned. \
                                              z = how far from the snare, y = height above snare centre."),
                            ("CP1 handle 1", "First Bézier control point. Controls curvature near the top."),
                            ("CP2 handle 2", "Second Bézier control point. Controls curvature near the bottom."),
                            ("P3  end",      "Bottom exit point — where the marble leaves the chute."),
                            ("Straight line","Collapses CP1/CP2 onto the P0–P3 line; pure ramp mode."),
                            ("Show handles", "Toggle the visible drag handles in the 3D viewport."),
                            ("Marble collis.","Enable physical marble–marble interaction \
                                              (off by default for cleaner single-marble analysis)."),
                            ("Reset",        "Restore all Bézier points to their factory defaults."),
                        ];
                        for (key, desc) in rows {
                            ui.label(egui::RichText::new(*key).strong());
                            ui.label(*desc);
                            ui.end_row();
                        }
                    });
                ui.add_space(2.0);
                ui.label("Tip: drag the coloured handles directly in the 3D view for intuitive \
                          curve editing. The chute mesh updates in real time.");
            });

        ui.add_space(4.0);

        // ── Pivot Arm ────────────────────────────────────────────────────────
        egui::CollapsingHeader::new(egui::RichText::new("Pivot Arm").strong())
            .id_source("help_pivot")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(
                    "The snare drum is mounted on a counterweighted pivot arm. \
                     The arm angle (Pivot θ in the Stats panel) determines the \
                     tilt of the snare head, which affects AoA."
                );
                ui.add_space(2.0);
                egui::Grid::new("help_pivot_grid")
                    .num_columns(2)
                    .spacing([12.0, 3.0])
                    .show(ui, |ui| {
                        let rows: &[(&str, &str)] = &[
                            ("θ < 0",  "Snare-side of the arm is lower (snare tilted toward chute)."),
                            ("θ = 0",  "Arm level; snare head is horizontal."),
                            ("θ > 0",  "Counterweight side is lower; snare tilted away from chute."),
                            ("Stops",  "Physical stop posts prevent the arm rotating beyond ±15–17°."),
                        ];
                        for (k, v) in rows {
                            ui.label(egui::RichText::new(*k).monospace().strong());
                            ui.label(*v);
                            ui.end_row();
                        }
                    });
            });

        ui.add_space(4.0);

        // ── Summary Stats ────────────────────────────────────────────────────
        egui::CollapsingHeader::new(egui::RichText::new("Summary Statistics").strong())
            .id_source("help_summary")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("help_summary_grid")
                    .num_columns(2)
                    .spacing([12.0, 3.0])
                    .show(ui, |ui| {
                        let rows: &[(&str, &str)] = &[
                            ("n",       "Number of complete runs (both marbles reached the snare)."),
                            ("mean",    "Arithmetic average over all complete runs."),
                            ("σ (std)", "Sample standard deviation (divides by n − 1). \
                                         Measures spread / consistency. Shown as ± after the mean."),
                            ("[a … b]", "Min and max observed values (range). Only shown for Δt."),
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

        // ── Graph window ─────────────────────────────────────────────────────
        egui::CollapsingHeader::new(egui::RichText::new("Velocity Graph").strong())
            .id_source("help_graph")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(
                    "Each run records the chute marble's velocity every frame. \
                     Open the graph for any run via the \"Show Graph\" button \
                     inside that run's entry in the Runs list."
                );
                ui.add_space(2.0);
                egui::Grid::new("help_graph_grid")
                    .num_columns(2)
                    .spacing([12.0, 3.0])
                    .show(ui, |ui| {
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
                          or the \"Hide Graph\" toggle in the run entry.");
            });
    });
}
