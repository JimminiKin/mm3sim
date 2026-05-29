use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::carousel::{spawn_carousel, CarouselPart};
use crate::components::chute::{spawn_chute, ChuteSegment};
use crate::components::hihat::{spawn_hihat, HiHatPart};
use crate::components::kick::{spawn_kick, KickPart};
use crate::components::ride::{spawn_ride, RidePart};
use crate::components::snare::{spawn_snare, PivotArm, SnarePart};
use crate::components::vibraphone::{spawn_vibraphone, VibraphoneEntity};
use crate::resources::carousel_params::{CarouselParams, CarouselState};
use crate::resources::chute_params::{ChuteParams, MultiChuteConfig, N_CHUTES};
use crate::resources::hihat_params::{HiHatParams, HiHatState};
use crate::resources::kick_params::KickParams;
use crate::resources::marble_collisions::MarbleCollisions;
use crate::resources::marble_params::MarbleParams;
use crate::resources::ride_params::RideParams;
use crate::resources::snare_params::SnareParams;
use crate::resources::stats_intake::StatsIntake;
use crate::resources::vibraphone_params::VibraphoneParams;
use crate::systems::sound::SnareVolume;

#[derive(Resource, Default)]
pub struct SnareFixed(pub bool);

pub fn apply_snare_fixed_system(
    mut commands: Commands,
    snare_fixed: Res<SnareFixed>,
    mut arm: Query<(Entity, &mut LinearVelocity, &mut AngularVelocity), With<PivotArm>>,
) {
    if !snare_fixed.is_changed() {
        return;
    }
    let Ok((entity, mut lin_vel, mut ang_vel)) = arm.single_mut() else {
        return;
    };
    if snare_fixed.0 {
        commands.entity(entity).insert(RigidBody::Static);
        *lin_vel = LinearVelocity::ZERO;
        *ang_vel = AngularVelocity::ZERO;
    } else {
        commands.entity(entity).insert(RigidBody::Dynamic);
    }
}

pub fn chute_editor_ui(
    mut contexts: EguiContexts,
    mut params: ResMut<ChuteParams>,
    mut multi: ResMut<MultiChuteConfig>,
    mut snare_params: ResMut<SnareParams>,
    mut vib: ResMut<VibraphoneParams>,
    mut hihat_params: ResMut<HiHatParams>,
    mut kick_params: ResMut<KickParams>,
    mut ride_params: ResMut<RideParams>,
    mut carousel_params: ResMut<CarouselParams>,
    carousel_state: Res<CarouselState>,
    mut marble_col: ResMut<MarbleCollisions>,
    mut marble_params: ResMut<MarbleParams>,
    mut stats_intake: ResMut<StatsIntake>,
    mut snare_fixed: ResMut<SnareFixed>,
    mut snare_volume: ResMut<SnareVolume>,
) {
    let ctx = contexts.ctx_mut().expect("primary egui context");
    egui::Window::new("Parameters")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new(egui::RichText::new("Parameters").strong())
                .id_salt("params_header")
                .default_open(true)
                .show(ui, |ui| {

                    // ── Options ──────────────────────────────────────────────
                    ui.heading("Options");
                    let old_col = marble_col.bypass_change_detection().0;
                    let mut new_col = old_col;
                    ui.checkbox(&mut new_col, "Marble-marble collisions");
                    if new_col != old_col {
                        marble_col.0 = new_col;
                    }

                    let old_si = stats_intake.bypass_change_detection().0;
                    let mut new_si = old_si;
                    ui.checkbox(&mut new_si, "Stats intake (graphs & ghosts)");
                    if new_si != old_si {
                        stats_intake.0 = new_si;
                    }

                    ui.horizontal(|ui| {
                        ui.label("Volume:");
                        ui.add(egui::Slider::new(&mut snare_volume.0, 0.0..=1.0).show_value(false));
                        ui.monospace(format!("{:.0}%", snare_volume.0 * 100.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Marble ⌀:");
                        let mut diameter_mm = marble_params.radius * 2000.0;
                        if ui.add(
                            egui::DragValue::new(&mut diameter_mm)
                                .speed(0.1)
                                .range(5.0_f32..=50.0_f32)
                        ).changed() {
                            marble_params.set_radius(diameter_mm / 2000.0);
                        }
                        ui.monospace(format!("mm  {:.2} g", marble_params.mass * 1000.0));
                    });

                    ui.separator();
                    if ui.button("Copy params as consts").clicked() {
                        let text = format_params_as_consts(
                            &params, &multi, &snare_params, &vib,
                            &hihat_params, &kick_params, &ride_params,
                        );
                        ui.ctx().copy_text(text);
                    }

                    // ── Ghost Snare ──────────────────────────────────────────
                    egui::CollapsingHeader::new("Ghost Snare")
                        .id_salt("ghost_snare_header")
                        .default_open(false)
                        .show(ui, |ui| {
                            let mut changed = false;

                            sub_heading(ui, "Shape");
                            changed |= point_drag_row(ui, "Exit end", &mut params.exit_pos);
                            changed |= scalar_drag_row(ui, "Exit length (m)", &mut params.exit_length, 0.001, 0.005..=0.50);
                            changed |= angle_drag_row(ui, "Exit angle (°)", &mut params.exit_angle, 0.0..=45.0);
                            changed |= scalar_drag_row(ui, "Curve radius (m)", &mut params.curve_radius, 0.001, 0.005..=1.0);
                            changed |= angle_drag_row(ui, "Slope angle (°)", &mut params.slope_angle, 1.0..=85.0);
                            changed |= scalar_drag_row(ui, "Slope length (m)", &mut params.slope_length, 0.001, 0.01..=1.0);

                            sub_heading(ui, "Surface");
                            changed |= scalar_drag_row(ui, "Restitution", &mut params.restitution, 0.01, 0.0..=1.0);
                            changed |= scalar_drag_row(ui, "Friction", &mut params.friction, 0.01, 0.0..=1.0);

                            sub_heading(ui, &format!("Angles ({N_CHUTES} chutes)"));
                            let mut angle_changed = false;
                            for i in 0..N_CHUTES {
                                angle_changed |= scalar_drag_row(
                                    ui,
                                    &format!("Chute {} (°)", i + 1),
                                    &mut multi.angles_deg[i],
                                    0.1,
                                    -180.0..=180.0,
                                );
                            }
                            ui.horizontal(|ui| {
                                if ui.small_button("Space 3°").clicked() {
                                    for i in 0..N_CHUTES {
                                        multi.angles_deg[i] = 3.0 + i as f32 * 3.0;
                                    }
                                    angle_changed = true;
                                }
                                if ui.small_button("Reset angles").clicked() {
                                    multi.angles_deg = MultiChuteConfig::default().angles_deg;
                                    angle_changed = true;
                                }
                            });
                            if angle_changed {
                                multi.dirty = true;
                            }

                            ui.separator();
                            if ui.button("Reset to defaults").clicked() {
                                *params = ChuteParams::default();
                                changed = true;
                            }

                            if changed {
                                params.dirty = true;
                            }
                        });

                    // ── Snare ────────────────────────────────────────────────
                    egui::CollapsingHeader::new("Snare")
                        .id_salt("snare_header")
                        .default_open(true)
                        .show(ui, |ui| {
                            let mut snare_changed = false;

                            sub_heading(ui, "Position");
                            snare_changed |= scalar_drag_row(ui, "X (m)", &mut snare_params.pos.x, 0.001, -1.0..=1.0);
                            snare_changed |= scalar_drag_row(ui, "Y (m)", &mut snare_params.pos.y, 0.001, -1.0..=1.0);
                            snare_changed |= scalar_drag_row(ui, "Z (m)", &mut snare_params.pos.z, 0.001, -1.0..=1.0);
                            if ui.button("Reset position").clicked() {
                                snare_params.pos = SnareParams::default().pos;
                                snare_changed = true;
                            }

                            sub_heading(ui, "Pivot");
                            let old_fixed = snare_fixed.bypass_change_detection().0;
                            let mut new_fixed = old_fixed;
                            ui.checkbox(&mut new_fixed, "Fix arm (freeze)");
                            if new_fixed != old_fixed {
                                snare_fixed.0 = new_fixed;
                            }

                            sub_heading(ui, "Surface");
                            snare_changed |= scalar_drag_row(ui, "Restitution", &mut snare_params.restitution, 0.01, 0.0..=1.0);
                            snare_changed |= scalar_drag_row(ui, "Friction", &mut snare_params.friction, 0.01, 0.0..=1.0);

                            if snare_changed {
                                snare_params.dirty = true;
                            }
                        });

                    // ── Vibraphone ───────────────────────────────────────────
                    egui::CollapsingHeader::new("Vibraphone")
                        .id_salt("vib_header")
                        .default_open(false)
                        .show(ui, |ui| {
                            let mut vib_changed = false;

                            sub_heading(ui, "Position");
                            vib_changed |= scalar_drag_row(ui, "X center (m)", &mut vib.pos.x, 0.001, -2.0..=2.0);
                            vib_changed |= scalar_drag_row(ui, "Y top face (m)", &mut vib.pos.y, 0.001, -0.5..=0.5);
                            vib_changed |= scalar_drag_row(ui, "Z (m)", &mut vib.pos.z, 0.001, -2.0..=0.0);

                            sub_heading(ui, "Bar geometry");
                            vib_changed |= scalar_drag_row(ui, "Width (m)", &mut vib.bar_width, 0.0005, 0.010..=0.10);
                            vib_changed |= scalar_drag_row(ui, "Spacing (m)", &mut vib.bar_spacing, 0.0005, 0.010..=0.20);
                            vib_changed |= scalar_drag_row(ui, "Thickness (m)", &mut vib.bar_thickness, 0.0005, 0.003..=0.05);
                            vib_changed |= scalar_drag_row(ui, "Length max (m)", &mut vib.bar_length_max, 0.001, 0.05..=0.80);
                            vib_changed |= scalar_drag_row(ui, "Length min (m)", &mut vib.bar_length_min, 0.001, 0.05..=0.50);
                            vib_changed |= scalar_drag_row(ui, "Density (kg/m³)", &mut vib.bar_density, 1.0, 500.0..=8000.0);

                            sub_heading(ui, "Surface");
                            vib_changed |= scalar_drag_row(ui, "Restitution", &mut vib.restitution, 0.01, 0.0..=1.0);
                            vib_changed |= scalar_drag_row(ui, "Friction", &mut vib.friction, 0.01, 0.0..=1.0);
                            vib_changed |= scalar_drag_row(ui, "Ang. damping", &mut vib.angular_damping, 0.01, 0.0..=20.0);

                            sub_heading(ui, "Pivot");
                            vib_changed |= scalar_drag_row(ui, "Arm scale (×len)", &mut vib.arm_scale, 0.01, 0.5..=4.0);
                            vib_changed |= scalar_drag_row(ui, "Pivot frac (×len)", &mut vib.pivot_frac, 0.005, 0.05..=0.48);
                            vib_changed |= angle_drag_row(ui, "Rest angle (°)", &mut vib.rest_deg, 1.0..=45.0);
                            vib_changed |= angle_drag_row(ui, "Max tilt (°)", &mut vib.max_tilt_deg, 0.5..=30.0);
                            vib_changed |= scalar_drag_row(ui, "CW ratio", &mut vib.cw_weight_ratio, 0.001, 0.5..=2.0);

                            if vib_changed {
                                vib.dirty = true;
                            }
                            ui.separator();
                            if ui.button("Reset vibraphone").clicked() {
                                *vib = VibraphoneParams::default();
                                vib.dirty = true;
                            }
                        });

                    // ── Hi-hat ───────────────────────────────────────────────
                    egui::CollapsingHeader::new("Hi-hat")
                        .id_salt("hihat_header")
                        .default_open(false)
                        .show(ui, |ui| {
                            let mut changed = false;

                            sub_heading(ui, "Position");
                            changed |= scalar_drag_row(ui, "X (m)", &mut hihat_params.pos.x, 0.001, -2.0..=2.0);
                            changed |= scalar_drag_row(ui, "Y (m)", &mut hihat_params.pos.y, 0.001, -1.0..=1.0);
                            changed |= scalar_drag_row(ui, "Z (m)", &mut hihat_params.pos.z, 0.001, -1.0..=1.0);
                            if ui.button("Reset position").clicked() {
                                hihat_params.pos = HiHatParams::default().pos;
                                changed = true;
                            }

                            sub_heading(ui, "Surface");
                            changed |= scalar_drag_row(ui, "Restitution", &mut hihat_params.restitution, 0.01, 0.0..=1.0);
                            changed |= scalar_drag_row(ui, "Friction", &mut hihat_params.friction, 0.01, 0.0..=1.0);

                            sub_heading(ui, "Gap");
                            changed |= scalar_drag_row(ui, "Open gap (m)", &mut hihat_params.gap_open, 0.001, 0.001..=0.10);
                            changed |= scalar_drag_row(ui, "Closed gap (m)", &mut hihat_params.gap_closed, 0.001, 0.0..=0.02);

                            if changed {
                                hihat_params.dirty = true;
                            }
                        });

                    // ── Kick ─────────────────────────────────────────────────
                    egui::CollapsingHeader::new("Kick")
                        .id_salt("kick_header")
                        .default_open(false)
                        .show(ui, |ui| {
                            let mut changed = false;

                            sub_heading(ui, "Position");
                            changed |= scalar_drag_row(ui, "X (m)", &mut kick_params.pos.x, 0.001, -2.0..=2.0);
                            changed |= scalar_drag_row(ui, "Y (m)", &mut kick_params.pos.y, 0.001, -1.0..=1.0);
                            changed |= scalar_drag_row(ui, "Z (m)", &mut kick_params.pos.z, 0.001, -1.0..=1.0);
                            if ui.button("Reset position").clicked() {
                                kick_params.pos = KickParams::default().pos;
                                changed = true;
                            }

                            sub_heading(ui, "Pivot");
                            changed |= scalar_drag_row(ui, "Rest deg", &mut kick_params.rest_deg, 0.1, 0.0..=45.0);
                            changed |= scalar_drag_row(ui, "Max tilt deg", &mut kick_params.max_tilt_deg, 0.1, 0.0..=10.0);
                            changed |= scalar_drag_row(ui, "Angular damping", &mut kick_params.angular_damping, 0.01, 0.0..=5.0);
                            changed |= scalar_drag_row(ui, "CW weight ratio", &mut kick_params.cw_weight_ratio, 0.01, 0.5..=3.0);

                            sub_heading(ui, "Surface");
                            changed |= scalar_drag_row(ui, "Restitution", &mut kick_params.restitution, 0.01, 0.0..=1.0);
                            changed |= scalar_drag_row(ui, "Friction", &mut kick_params.friction, 0.01, 0.0..=1.0);

                            if changed {
                                kick_params.dirty = true;
                            }
                        });

                    // ── Ride ─────────────────────────────────────────────────
                    egui::CollapsingHeader::new("Ride")
                        .id_salt("ride_header")
                        .default_open(false)
                        .show(ui, |ui| {
                            let mut changed = false;

                            sub_heading(ui, "Position");
                            changed |= scalar_drag_row(ui, "X (m)", &mut ride_params.pos.x, 0.001, -2.0..=2.0);
                            changed |= scalar_drag_row(ui, "Y (m)", &mut ride_params.pos.y, 0.001, -1.0..=1.0);
                            changed |= scalar_drag_row(ui, "Z (m)", &mut ride_params.pos.z, 0.001, -1.0..=1.0);
                            if ui.button("Reset position").clicked() {
                                ride_params.pos = RideParams::default().pos;
                                changed = true;
                            }

                            sub_heading(ui, "Surface");
                            changed |= scalar_drag_row(ui, "Restitution", &mut ride_params.restitution, 0.01, 0.0..=1.0);
                            changed |= scalar_drag_row(ui, "Friction", &mut ride_params.friction, 0.01, 0.0..=1.0);

                            if changed {
                                ride_params.dirty = true;
                            }
                        });

                    // ── Carousel ─────────────────────────────────────────────
                    egui::CollapsingHeader::new("Carousel")
                        .id_salt("carousel_header")
                        .default_open(false)
                        .show(ui, |ui| {
                            let mut changed = false;

                            let slot_names = ["Crash (0)", "Cowbell (1)", "Tambourine (2)", "Woodblock (3)"];
                            ui.label(format!(
                                "Current slot: {} — {}",
                                carousel_state.current_slot,
                                slot_names[carousel_state.current_slot as usize]
                            ));
                            if carousel_state.is_animating {
                                ui.label("Rotating…");
                            }

                            sub_heading(ui, "Position");
                            changed |= scalar_drag_row(ui, "X (m)", &mut carousel_params.pos.x, 0.001, -3.0..=3.0);
                            changed |= scalar_drag_row(ui, "Y (m)", &mut carousel_params.pos.y, 0.001, -1.0..=1.0);
                            changed |= scalar_drag_row(ui, "Z (m)", &mut carousel_params.pos.z, 0.001, -1.0..=1.0);
                            if ui.button("Reset position").clicked() {
                                carousel_params.pos = CarouselParams::default().pos;
                                changed = true;
                            }

                            sub_heading(ui, "Instrument tilt");
                            changed |= angle_drag_row(ui, "Tilt (°)", &mut carousel_params.tilt_deg, -90.0..=90.0);
                            ui.label(egui::RichText::new(
                                "−90°–0° = face angled toward dropper · 0° = face-up · 90° = on edge"
                            ).weak().small());
                            if ui.small_button("Reset tilt").clicked() {
                                carousel_params.tilt_deg = CarouselParams::default().tilt_deg;
                                changed = true;
                            }

                            sub_heading(ui, "Crash cymbal (slot 0)");
                            changed |= scalar_drag_row(ui, "Restitution", &mut carousel_params.crash_restitution, 0.01, 0.0..=1.0);
                            changed |= scalar_drag_row(ui, "Friction", &mut carousel_params.crash_friction, 0.01, 0.0..=1.0);

                            sub_heading(ui, "Cowbell (slot 1)");
                            changed |= scalar_drag_row(ui, "Restitution", &mut carousel_params.cowbell_restitution, 0.01, 0.0..=1.0);
                            changed |= scalar_drag_row(ui, "Friction", &mut carousel_params.cowbell_friction, 0.01, 0.0..=1.0);

                            sub_heading(ui, "Tambourine (slot 2)");
                            changed |= scalar_drag_row(ui, "Restitution", &mut carousel_params.tamb_restitution, 0.01, 0.0..=1.0);
                            changed |= scalar_drag_row(ui, "Friction", &mut carousel_params.tamb_friction, 0.01, 0.0..=1.0);

                            sub_heading(ui, "Woodblock (slot 3)");
                            changed |= scalar_drag_row(ui, "Restitution", &mut carousel_params.wood_restitution, 0.01, 0.0..=1.0);
                            changed |= scalar_drag_row(ui, "Friction", &mut carousel_params.wood_friction, 0.01, 0.0..=1.0);

                            if changed {
                                carousel_params.dirty = true;
                            }
                        });

                });
        });
}

fn sub_heading(ui: &mut egui::Ui, text: &str) {
    ui.separator();
    ui.label(egui::RichText::new(text).strong());
}

fn point_drag_row(ui: &mut egui::Ui, label: &str, pt: &mut [f32; 2]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed |= ui
                .add(egui::DragValue::new(&mut pt[1]).prefix("y ").speed(0.001))
                .changed();
            changed |= ui
                .add(egui::DragValue::new(&mut pt[0]).prefix("z ").speed(0.001))
                .changed();
        });
    });
    changed
}

fn scalar_drag_row(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut f32,
    speed: f64,
    range: std::ops::RangeInclusive<f32>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed = ui
                .add(egui::DragValue::new(val).speed(speed).range(range))
                .changed();
        });
    });
    changed
}

fn angle_drag_row(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed = ui
                .add(egui::DragValue::new(val).speed(0.1).range(range))
                .changed();
        });
    });
    changed
}

fn fmt_f32(v: f32) -> String {
    let s = format!("{v}");
    if s.contains('.') || s.contains('e') { s } else { format!("{s}.0") }
}

fn format_params_as_consts(
    params: &ChuteParams,
    multi: &MultiChuteConfig,
    snare: &SnareParams,
    vib: &VibraphoneParams,
    hihat: &HiHatParams,
    kick: &KickParams,
    ride: &RideParams,
) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let f = fmt_f32;

    writeln!(s, "// All tunable physics/geometry parameters exposed in the Parameters panel.").unwrap();
    writeln!(s, "// Paste the output of \"Copy params as consts\" to fully replace this file.").unwrap();
    writeln!(s).unwrap();

    writeln!(s, "// ── Ghost Snare ───────────────────────────────────────────────────────────────").unwrap();
    writeln!(s, "pub const CHUTE_EXIT_Z: f32 = {};", f(params.exit_pos[0])).unwrap();
    writeln!(s, "pub const CHUTE_EXIT_Y: f32 = {};", f(params.exit_pos[1])).unwrap();
    writeln!(s, "pub const CHUTE_EXIT_LENGTH: f32 = {};", f(params.exit_length)).unwrap();
    writeln!(s, "pub const CHUTE_EXIT_ANGLE: f32 = {};", f(params.exit_angle)).unwrap();
    writeln!(s, "pub const CHUTE_CURVE_RADIUS: f32 = {};", f(params.curve_radius)).unwrap();
    writeln!(s, "pub const CHUTE_SLOPE_ANGLE: f32 = {};", f(params.slope_angle)).unwrap();
    writeln!(s, "pub const CHUTE_SLOPE_LENGTH: f32 = {};", f(params.slope_length)).unwrap();
    writeln!(s, "pub const CHUTE_RESTITUTION: f32 = {};", f(params.restitution)).unwrap();
    writeln!(s, "pub const CHUTE_FRICTION: f32 = {};", f(params.friction)).unwrap();
    let angles: Vec<String> = multi.angles_deg.iter().map(|&a| f(a)).collect();
    writeln!(s, "pub const CHUTE_ANGLES: [f32; 6] = [{}];", angles.join(", ")).unwrap();
    writeln!(s).unwrap();

    writeln!(s, "// ── Snare ─────────────────────────────────────────────────────────────────────").unwrap();
    writeln!(s, "pub const SNARE_RESTITUTION: f32 = {};", f(snare.restitution)).unwrap();
    writeln!(s, "pub const SNARE_FRICTION: f32 = {};", f(snare.friction)).unwrap();
    writeln!(s, "pub const SNARE_POS_X: f32 = {};", f(snare.pos.x)).unwrap();
    writeln!(s, "pub const SNARE_POS_Y: f32 = {};", f(snare.pos.y)).unwrap();
    writeln!(s, "pub const SNARE_POS_Z: f32 = {};", f(snare.pos.z)).unwrap();
    writeln!(s).unwrap();

    writeln!(s, "// ── Vibraphone ────────────────────────────────────────────────────────────────").unwrap();
    writeln!(s, "pub const VIB_ROW_X: f32 = {};", f(vib.pos.x)).unwrap();
    writeln!(s, "pub const VIB_ROW_Y: f32 = {};", f(vib.pos.y)).unwrap();
    writeln!(s, "pub const VIB_ROW_Z: f32 = {};", f(vib.pos.z)).unwrap();
    writeln!(s, "pub const VIB_BAR_WIDTH: f32 = {};", f(vib.bar_width)).unwrap();
    writeln!(s, "pub const VIB_BAR_SPACING: f32 = {};", f(vib.bar_spacing)).unwrap();
    writeln!(s, "pub const VIB_BAR_THICKNESS: f32 = {};", f(vib.bar_thickness)).unwrap();
    writeln!(s, "pub const VIB_BAR_LENGTH_MAX: f32 = {};", f(vib.bar_length_max)).unwrap();
    writeln!(s, "pub const VIB_BAR_LENGTH_MIN: f32 = {};", f(vib.bar_length_min)).unwrap();
    writeln!(s, "pub const VIB_BAR_DENSITY: f32 = {};", f(vib.bar_density)).unwrap();
    writeln!(s, "pub const VIB_ANGULAR_DAMPING: f32 = {};", f(vib.angular_damping)).unwrap();
    writeln!(s, "pub const VIB_RESTITUTION: f32 = {};", f(vib.restitution)).unwrap();
    writeln!(s, "pub const VIB_FRICTION: f32 = {};", f(vib.friction)).unwrap();
    writeln!(s, "pub const VIB_ARM_SCALE: f32 = {};", f(vib.arm_scale)).unwrap();
    writeln!(s, "pub const VIB_PIVOT_FRAC: f32 = {};", f(vib.pivot_frac)).unwrap();
    writeln!(s, "pub const VIB_REST_DEG: f32 = {};", f(vib.rest_deg)).unwrap();
    writeln!(s, "pub const VIB_MAX_TILT_DEG: f32 = {};", f(vib.max_tilt_deg)).unwrap();
    writeln!(s, "pub const VIB_CW_WEIGHT_RATIO: f32 = {};", f(vib.cw_weight_ratio)).unwrap();
    writeln!(s).unwrap();

    writeln!(s, "// ── Hi-hat ────────────────────────────────────────────────────────────────────").unwrap();
    writeln!(s, "pub const HIHAT_X: f32 = {};", f(hihat.pos.x)).unwrap();
    writeln!(s, "pub const HIHAT_Y: f32 = {};", f(hihat.pos.y)).unwrap();
    writeln!(s, "pub const HIHAT_Z: f32 = {};", f(hihat.pos.z)).unwrap();
    writeln!(s, "pub const HIHAT_RESTITUTION: f32 = {};", f(hihat.restitution)).unwrap();
    writeln!(s, "pub const HIHAT_FRICTION: f32 = {};", f(hihat.friction)).unwrap();
    writeln!(s, "pub const HIHAT_GAP_OPEN: f32 = {};", f(hihat.gap_open)).unwrap();
    writeln!(s, "pub const HIHAT_GAP_CLOSED: f32 = {};", f(hihat.gap_closed)).unwrap();
    writeln!(s).unwrap();

    writeln!(s, "// ── Kick ──────────────────────────────────────────────────────────────────────").unwrap();
    writeln!(s, "pub const KICK_X: f32 = {};", f(kick.pos.x)).unwrap();
    writeln!(s, "pub const KICK_Y: f32 = {};", f(kick.pos.y)).unwrap();
    writeln!(s, "pub const KICK_Z: f32 = {};", f(kick.pos.z)).unwrap();
    writeln!(s, "pub const KICK_RESTITUTION: f32 = {};", f(kick.restitution)).unwrap();
    writeln!(s, "pub const KICK_FRICTION: f32 = {};", f(kick.friction)).unwrap();
    writeln!(s, "pub const KICK_REST_DEG: f32 = {};", f(kick.rest_deg)).unwrap();
    writeln!(s, "pub const KICK_MAX_TILT_DEG: f32 = {};", f(kick.max_tilt_deg)).unwrap();
    writeln!(s, "pub const KICK_ANGULAR_DAMPING: f32 = {};", f(kick.angular_damping)).unwrap();
    writeln!(s, "pub const KICK_CW_WEIGHT_RATIO: f32 = {};", f(kick.cw_weight_ratio)).unwrap();
    writeln!(s).unwrap();

    writeln!(s, "// ── Ride ──────────────────────────────────────────────────────────────────────").unwrap();
    writeln!(s, "pub const RIDE_X: f32 = {};", f(ride.pos.x)).unwrap();
    writeln!(s, "pub const RIDE_Y: f32 = {};", f(ride.pos.y)).unwrap();
    writeln!(s, "pub const RIDE_Z: f32 = {};", f(ride.pos.z)).unwrap();
    writeln!(s, "pub const RIDE_RESTITUTION: f32 = {};", f(ride.restitution)).unwrap();
    writeln!(s, "pub const RIDE_FRICTION: f32 = {};", f(ride.friction)).unwrap();

    s
}

/// Despawns all `SnarePart` entities and respawns the snare when `SnareParams.dirty` is set.
pub fn rebuild_snare_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<SnareParams>,
    entities: Query<Entity, With<SnarePart>>,
) {
    if !params.dirty {
        return;
    }
    params.dirty = false;
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    spawn_snare(&mut commands, &mut meshes, &mut materials, &params);
}

pub fn rebuild_vibraphone_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<VibraphoneParams>,
    entities: Query<Entity, With<VibraphoneEntity>>,
) {
    if !params.dirty {
        return;
    }
    params.dirty = false;
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    spawn_vibraphone(&mut commands, &mut meshes, &mut materials, &params);
}

pub fn rebuild_chute_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<ChuteParams>,
    mut multi: ResMut<MultiChuteConfig>,
    snare_params: Res<SnareParams>,
    segments: Query<Entity, With<ChuteSegment>>,
) {
    if !params.dirty && !multi.dirty && !snare_params.is_changed() {
        return;
    }
    params.dirty = false;
    multi.dirty = false;
    for entity in &segments {
        commands.entity(entity).despawn();
    }
    for i in 0..N_CHUTES {
        let angle_rad = multi.angles_deg[i].to_radians();
        spawn_chute(&mut commands, &mut meshes, &mut materials, &params, snare_params.pos, angle_rad);
    }
}

pub fn rebuild_hihat_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<HiHatParams>,
    state: Res<HiHatState>,
    entities: Query<Entity, With<HiHatPart>>,
) {
    if !params.dirty {
        return;
    }
    params.dirty = false;
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    spawn_hihat(&mut commands, &mut meshes, &mut materials, &params, state.open);
}

pub fn rebuild_kick_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<KickParams>,
    entities: Query<Entity, With<KickPart>>,
) {
    if !params.dirty {
        return;
    }
    params.dirty = false;
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    spawn_kick(&mut commands, &mut meshes, &mut materials, &params);
}

pub fn rebuild_ride_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<RideParams>,
    entities: Query<Entity, With<RidePart>>,
) {
    if !params.dirty {
        return;
    }
    params.dirty = false;
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    spawn_ride(&mut commands, &mut meshes, &mut materials, &params);
}

pub fn rebuild_carousel_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<CarouselParams>,
    mut state: ResMut<CarouselState>,
    entities: Query<Entity, With<CarouselPart>>,
) {
    if !params.dirty {
        return;
    }
    params.dirty = false;
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    // Reset rotation state on rebuild so the new assembly starts at slot 0 on top.
    *state = CarouselState::default();
    spawn_carousel(&mut commands, &mut meshes, &mut materials, &params);
}
