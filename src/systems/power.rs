//! Power Needs panel — energy and power budget for one programming-wheel revolution.
//!
//! Categories: Triggers (spring+actuator), Lifting (marble PE), Hi-hat close,
//! Selector (carousel), Wheel bearing (continuous).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_plot::{Bar, BarChart, Legend, Plot};

use crate::resources::marble_params::MarbleParams;
use crate::resources::power_params::PowerParams;
use crate::resources::programming_wheel_params::{channel_target, ChannelTarget, ProgrammingWheelParams};

const POWER_WINDOW_WIDTH: f32 = 430.0;
const GRAPH_HEIGHT:       f32 = 150.0;

const COL_TRIGGERS: (u8, u8, u8) = (220,  90,  50);
const COL_LIFTING:  (u8, u8, u8) = ( 60, 140, 220);
const COL_HIHAT:    (u8, u8, u8) = (200, 165,  40);
const COL_SELECTOR: (u8, u8, u8) = (160,  80, 220);
const COL_BEARING:  (u8, u8, u8) = (140, 140, 140);

// ── Trigger counting ─────────────────────────────────────────────────────────

#[derive(Default)]
struct TriggerCounts {
    ghost_snare:     usize,
    snare:           usize,
    vibraphone:      usize,
    hihat_strike:    usize,
    hihat_pedal:     usize,
    kick:            usize,
    ride:            usize,
    carousel_drop:   usize,
    carousel_select: usize,
}

impl TriggerCounts {
    fn total_marble_drops(&self) -> usize {
        self.ghost_snare + self.snare + self.vibraphone + self.hihat_strike
            + self.kick + self.ride + self.carousel_drop
    }
}

fn count_triggers(wheel: &ProgrammingWheelParams) -> TriggerCounts {
    let mut c = TriggerCounts::default();
    for note in &wheel.notes {
        match channel_target(note.channel) {
            ChannelTarget::GhostSnare      => c.ghost_snare     += 1,
            ChannelTarget::Snare { .. }    => c.snare           += 1,
            ChannelTarget::VibBar { .. }   => c.vibraphone      += 1,
            ChannelTarget::HiHat { .. }    => c.hihat_strike    += 1,
            ChannelTarget::HiHatPedal      => c.hihat_pedal     += 1,
            ChannelTarget::Kick { .. }     => c.kick            += 1,
            ChannelTarget::Ride { .. }     => c.ride            += 1,
            ChannelTarget::Carousel { .. } => c.carousel_drop   += 1,
            ChannelTarget::CarouselSelect  => c.carousel_select += 1,
        }
    }
    c
}

// ── Energy formulas (all return mJ) ─────────────────────────────────────────

fn e_spring_mj(k: f32, x_cm: f32) -> f32 {
    let x = x_cm / 100.0;
    0.5 * k * x * x * 1000.0
}

// Peak KE for uniform accel from rest: v_peak = 2d/t
fn e_actuator_mj(mass_g: f32, travel_cm: f32, time_ms: f32) -> f32 {
    let m = mass_g / 1000.0;
    let d = travel_cm / 100.0;
    let t = (time_ms / 1000.0).max(1e-6);
    let v_peak = 2.0 * d / t;
    0.5 * m * v_peak * v_peak * 1000.0
}

fn e_actuation_mj(p: &PowerParams) -> f32 {
    e_spring_mj(p.spring_k_n_per_m, p.spring_compression_cm)
        + e_actuator_mj(p.actuator_mass_g, p.actuator_travel_cm, p.actuator_time_ms)
}

fn e_lift_mj(height_m: f32, marble_mass: f32) -> f32 {
    marble_mass * 9.81 * height_m * 1000.0
}

fn e_hihat_close_mj(p: &PowerParams) -> f32 {
    (p.hihat_close_mass_g / 1000.0) * 9.81 * (p.hihat_close_travel_cm / 100.0) * 1000.0
}

fn e_carousel_sel_mj(p: &PowerParams) -> f32 {
    0.5 * p.carousel_inertia_kg_m2 * p.carousel_omega_rad_s * p.carousel_omega_rad_s * 1000.0
}

// W → mW: μ × m × g × r_bearing × ω_wheel
fn p_bearing_mw(p: &PowerParams, rpm: f32) -> f32 {
    let r     = p.bearing_radius_mm / 1000.0;
    let omega = rpm * std::f32::consts::TAU / 60.0;
    p.bearing_friction_mu * p.wheel_mass_kg * 9.81 * r * omega * 1000.0
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn col32(c: (u8, u8, u8)) -> egui::Color32 {
    egui::Color32::from_rgb(c.0, c.1, c.2)
}

fn mj_to_mw(mj_per_rev: f32, rpm: f32) -> f32 {
    mj_per_rev * rpm / 60.0
}

// ── Sub-panels ───────────────────────────────────────────────────────────────

fn render_params(ui: &mut egui::Ui, p: &mut PowerParams) {
    egui::Grid::new("power_params_grid")
        .num_columns(3)
        .spacing([8.0, 2.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Machine height").strong());
            ui.add(egui::DragValue::new(&mut p.machine_height_m).speed(0.05).range(0.1_f32..=15.0_f32));
            ui.label("m");
            ui.end_row();

            ui.separator(); ui.separator(); ui.separator(); ui.end_row();
            ui.label(egui::RichText::new("Trigger actuation").strong());
            ui.label(""); ui.label(""); ui.end_row();

            ui.label("  Spring k");
            ui.add(egui::DragValue::new(&mut p.spring_k_n_per_m).speed(1.0).range(1.0_f32..=5000.0_f32));
            ui.label("N/m");
            ui.end_row();

            ui.label("  Spring compression");
            ui.add(egui::DragValue::new(&mut p.spring_compression_cm).speed(0.05).range(0.0_f32..=10.0_f32));
            ui.label("cm");
            ui.end_row();

            ui.label("  Actuator mass");
            ui.add(egui::DragValue::new(&mut p.actuator_mass_g).speed(0.5).range(1.0_f32..=500.0_f32));
            ui.label("g");
            ui.end_row();

            ui.label("  Actuator travel");
            ui.add(egui::DragValue::new(&mut p.actuator_travel_cm).speed(0.05).range(0.1_f32..=10.0_f32));
            ui.label("cm");
            ui.end_row();

            ui.label("  Actuator time");
            ui.add(egui::DragValue::new(&mut p.actuator_time_ms).speed(1.0).range(1.0_f32..=500.0_f32));
            ui.label("ms");
            ui.end_row();

            ui.separator(); ui.separator(); ui.separator(); ui.end_row();
            ui.label(egui::RichText::new("Hi-hat close").strong());
            ui.label(""); ui.label(""); ui.end_row();

            ui.label("  Mechanism mass");
            ui.add(egui::DragValue::new(&mut p.hihat_close_mass_g).speed(1.0).range(1.0_f32..=500.0_f32));
            ui.label("g");
            ui.end_row();

            ui.label("  Cymbal travel");
            ui.add(egui::DragValue::new(&mut p.hihat_close_travel_cm).speed(0.05).range(0.01_f32..=10.0_f32));
            ui.label("cm");
            ui.end_row();

            ui.separator(); ui.separator(); ui.separator(); ui.end_row();
            ui.label(egui::RichText::new("Selector (carousel)").strong());
            ui.label(""); ui.label(""); ui.end_row();

            ui.label("  Inertia");
            ui.add(egui::DragValue::new(&mut p.carousel_inertia_kg_m2).speed(0.0005).range(0.0001_f32..=1.0_f32));
            ui.label("kg·m²");
            ui.end_row();

            ui.label("  Index speed");
            ui.add(egui::DragValue::new(&mut p.carousel_omega_rad_s).speed(0.1).range(0.1_f32..=50.0_f32));
            ui.label("rad/s");
            ui.end_row();

            ui.separator(); ui.separator(); ui.separator(); ui.end_row();
            ui.label(egui::RichText::new("Wheel bearing").strong());
            ui.label(""); ui.label(""); ui.end_row();

            ui.label("  Wheel mass");
            ui.add(egui::DragValue::new(&mut p.wheel_mass_kg).speed(0.05).range(0.1_f32..=20.0_f32));
            ui.label("kg");
            ui.end_row();

            ui.label("  Friction coeff");
            ui.add(egui::DragValue::new(&mut p.bearing_friction_mu).speed(0.0001).range(0.00001_f32..=0.1_f32));
            ui.label("μ");
            ui.end_row();

            ui.label("  Bore radius");
            ui.add(egui::DragValue::new(&mut p.bearing_radius_mm).speed(0.1).range(1.0_f32..=50.0_f32));
            ui.label("mm");
            ui.end_row();
        });
}

fn render_instrument_detail(
    ui: &mut egui::Ui,
    counts: &TriggerCounts,
    e_per_marble_mj: f32,
    e_hh_mj: f32,
    e_sel_mj: f32,
    rpm: f32,
) {
    let marble_rows: &[(&str, usize, (u8, u8, u8))] = &[
        ("Ghost Snare",  counts.ghost_snare,   (51,  115, 230)),
        ("Snare",        counts.snare,         (242,  89,  38)),
        ("Vibraphone",   counts.vibraphone,    ( 80, 200, 120)),
        ("Hi-Hat",       counts.hihat_strike,  (200, 165,  40)),
        ("Kick",         counts.kick,          (180, 110,  55)),
        ("Ride",         counts.ride,          (210, 175,  60)),
        ("Carousel",     counts.carousel_drop, (160,  80, 220)),
    ];

    egui::Grid::new("power_instr_grid")
        .num_columns(4)
        .spacing([8.0, 2.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Instrument").weak());
            ui.label(egui::RichText::new("n/rev").weak());
            ui.label(egui::RichText::new("mJ/rev").weak());
            ui.label(egui::RichText::new("mW").weak());
            ui.end_row();

            for (name, n, col) in marble_rows {
                if *n == 0 { continue; }
                let mj = *n as f32 * e_per_marble_mj;
                ui.label(egui::RichText::new(*name).color(col32(*col)));
                ui.monospace(format!("{:4}", n));
                ui.monospace(format!("{:.1}", mj));
                ui.monospace(format!("{:.3}", mj_to_mw(mj, rpm)));
                ui.end_row();
            }

            if counts.hihat_pedal > 0 {
                let mj = counts.hihat_pedal as f32 * e_hh_mj;
                ui.label(egui::RichText::new("HH pedal").color(col32(COL_HIHAT)));
                ui.monospace(format!("{:4}", counts.hihat_pedal));
                ui.monospace(format!("{:.2}", mj));
                ui.monospace(format!("{:.4}", mj_to_mw(mj, rpm)));
                ui.end_row();
            }

            if counts.carousel_select > 0 {
                let mj = counts.carousel_select as f32 * e_sel_mj;
                ui.label(egui::RichText::new("Carousel sel").color(col32(COL_SELECTOR)));
                ui.monospace(format!("{:4}", counts.carousel_select));
                ui.monospace(format!("{:.2}", mj));
                ui.monospace(format!("{:.4}", mj_to_mw(mj, rpm)));
                ui.end_row();
            }
        });
}

// ── Main system ──────────────────────────────────────────────────────────────

pub fn power_panel_ui(
    mut contexts: EguiContexts,
    mut params: ResMut<PowerParams>,
    wheel: Res<ProgrammingWheelParams>,
    marble_params: Res<MarbleParams>,
) {
    let ctx = contexts.ctx_mut().expect("primary egui context");

    egui::Window::new("Power")
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-8.0, -8.0))
        .default_width(POWER_WINDOW_WIDTH)
        .resizable(true)
        .title_bar(false)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new(egui::RichText::new("Power Needs").strong())
                .id_salt("power_header")
                .default_open(true)
                .show(ui, |ui| {
                    let counts  = count_triggers(&wheel);
                    let rpm     = wheel.rpm;
                    let n_drops = counts.total_marble_drops();

                    // Per-event energies (mJ)
                    let e_act  = e_actuation_mj(&params);
                    let e_lift = e_lift_mj(params.machine_height_m, marble_params.mass);
                    let e_hh   = e_hihat_close_mj(&params);
                    let e_sel  = e_carousel_sel_mj(&params);
                    let p_bear = p_bearing_mw(&params, rpm);

                    // Budget categories: (label, count, mJ/event, color)
                    let budget: [(&str, usize, f32, (u8, u8, u8)); 4] = [
                        ("Triggers",  n_drops,               e_act,  COL_TRIGGERS),
                        ("Lifting",   n_drops,               e_lift, COL_LIFTING),
                        ("Hi-hat",    counts.hihat_pedal,    e_hh,   COL_HIHAT),
                        ("Selector",  counts.carousel_select, e_sel, COL_SELECTOR),
                    ];

                    let total_mj: f32 = budget.iter().map(|(_, n, e, _)| *n as f32 * e).sum();
                    let total_mw = mj_to_mw(total_mj, rpm) + p_bear;

                    // ── Power budget ───────────────────────────────────────
                    egui::Grid::new("power_budget_grid")
                        .num_columns(5)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("");
                            ui.label(egui::RichText::new("n/rev").weak().small());
                            ui.label(egui::RichText::new("mJ/event").weak().small());
                            ui.label(egui::RichText::new("mJ/rev").weak().small());
                            ui.label(egui::RichText::new("mW").weak().small());
                            ui.end_row();

                            for (label, n, e, col) in &budget {
                                let mj_rev = *n as f32 * e;
                                let mw     = mj_to_mw(mj_rev, rpm);
                                ui.label(egui::RichText::new(*label)
                                    .color(col32(*col)).strong());
                                ui.monospace(format!("{}", n));
                                ui.monospace(format!("{:.2}", e));
                                ui.monospace(format!("{:.1}", mj_rev));
                                ui.monospace(format!("{:.3}", mw));
                                ui.end_row();
                            }

                            ui.label(egui::RichText::new("Wheel")
                                .color(col32(COL_BEARING)));
                            ui.label(egui::RichText::new("—").weak());
                            ui.label(egui::RichText::new("—").weak());
                            ui.label(egui::RichText::new("—").weak());
                            ui.monospace(format!("{:.4}", p_bear));
                            ui.end_row();

                            ui.separator(); ui.separator(); ui.separator();
                            ui.separator(); ui.separator(); ui.end_row();

                            ui.label(egui::RichText::new("Total").strong());
                            ui.label("");
                            ui.label("");
                            ui.monospace(
                                egui::RichText::new(format!("{:.1}", total_mj)).strong()
                            );
                            ui.monospace(
                                egui::RichText::new(format!("{:.3}", total_mw)).strong()
                            );
                            ui.end_row();
                        });

                    ui.label(
                        egui::RichText::new(format!("@ {:.3} RPM", rpm)).weak().small()
                    );

                    // ── Trigger sub-breakdown hint ─────────────────────────
                    {
                        let e_spring = e_spring_mj(params.spring_k_n_per_m, params.spring_compression_cm);
                        let e_arm    = e_actuator_mj(params.actuator_mass_g, params.actuator_travel_cm, params.actuator_time_ms);
                        ui.label(
                            egui::RichText::new(
                                format!("  Triggers: spring {:.2} mJ  +  arm KE {:.2} mJ  =  {:.2} mJ",
                                    e_spring, e_arm, e_act)
                            ).weak().small()
                        );
                    }

                    // ── By instrument (collapsible) ────────────────────────
                    egui::CollapsingHeader::new("By instrument")
                        .id_salt("power_instr_header")
                        .default_open(false)
                        .show(ui, |ui| {
                            render_instrument_detail(
                                ui, &counts,
                                e_act + e_lift, e_hh, e_sel, rpm,
                            );
                        });

                    // ── Parameters (collapsible) ───────────────────────────
                    egui::CollapsingHeader::new("Parameters")
                        .id_salt("power_params_header")
                        .default_open(false)
                        .show(ui, |ui| render_params(ui, &mut params));

                    // ── Graph (mW per category) ────────────────────────────
                    ui.horizontal(|ui| {
                        let lbl = if params.show_graph { "Hide Graph" } else { "Show Graph" };
                        if ui.small_button(lbl).clicked() {
                            params.show_graph = !params.show_graph;
                        }
                    });

                    if params.show_graph {
                        let graph_bars: [(&str, f32, (u8, u8, u8)); 5] = [
                            ("Triggers",  mj_to_mw(budget[0].1 as f32 * e_act,  rpm), COL_TRIGGERS),
                            ("Lifting",   mj_to_mw(budget[1].1 as f32 * e_lift, rpm), COL_LIFTING),
                            ("Hi-hat",    mj_to_mw(counts.hihat_pedal    as f32 * e_hh,  rpm), COL_HIHAT),
                            ("Selector",  mj_to_mw(counts.carousel_select as f32 * e_sel, rpm), COL_SELECTOR),
                            ("Wheel",     p_bear,                                        COL_BEARING),
                        ];

                        Plot::new("power_mw_graph")
                            .height(GRAPH_HEIGHT)
                            .legend(Legend::default())
                            .y_axis_label("mW")
                            .allow_drag(false)
                            .allow_zoom(false)
                            .allow_scroll(false)
                            .show_axes([false, true])
                            .show(ui, |plot_ui| {
                                for (i, (name, mw, col)) in graph_bars.iter().enumerate() {
                                    let bar = Bar::new(i as f64, *mw as f64)
                                        .fill(col32(*col))
                                        .width(0.7);
                                    plot_ui.bar_chart(BarChart::new(*name, vec![bar]));
                                }
                            });
                    }
                });
        });
}
