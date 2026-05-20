use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::barrel::{spawn_barrel, BarrelCylinder};
use crate::components::snare::SnareDrum;
use crate::resources::barrel_params::{
    channel_color_rgb, channel_name, BarrelParams, BARREL_CH_CHUTE, BARREL_CH_DROP,
    BARREL_CH_VIB_FIRST,
};
use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;
use crate::resources::marble_collisions::MarbleCollisions;
use crate::resources::marble_runs::RunHistory;
use crate::resources::vibraphone_params::VibraphoneParams;
use crate::systems::marble::{
    jittered_spawn, spawn_chute_marble, spawn_marble, spawn_vib_marble_for_bar,
};

// ── Setup ─────────────────────────────────────────────────────────────────────

pub fn setup_barrel_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_barrel(&mut commands, &mut meshes, &mut materials);
}

// ── Rotation + step-crossing detection ────────────────────────────────────────
// Triggers are written to params.pending_spawns and consumed by barrel_spawn_system.

pub fn rotate_barrel_system(
    time: Res<Time>,
    mut params: ResMut<BarrelParams>,
    mut cylinder_q: Query<&mut Transform, With<BarrelCylinder>>,
) {
    // Always update the cylinder mesh rotation (visible even when paused).
    // Base orientation: rotation_z(π/2) aligns the Bevy cylinder (Y-axis) with world X.
    // Spin: rotation_y(−angle) in model space rotates it around that X axis.
    if let Ok(mut tf) = cylinder_q.single_mut() {
        let base = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let spin = Quat::from_rotation_y(-params.angle);
        tf.rotation = base * spin;
    }

    if !params.enabled {
        return;
    }

    let step_angle = std::f32::consts::TAU / BARREL_N_STEPS as f32;
    let prev_angle = params.angle;

    let delta = std::f32::consts::TAU * (params.rpm / 60.0) * time.delta_secs();
    params.angle = (params.angle + delta).rem_euclid(std::f32::consts::TAU);

    let prev_step = (prev_angle / step_angle) as usize % BARREL_N_STEPS;
    let curr_step = (params.angle / step_angle) as usize % BARREL_N_STEPS;
    params.current_step = curr_step;

    if curr_step == prev_step {
        return;
    }

    // Number of step boundaries crossed this frame (handles large dt gracefully)
    let n = if curr_step > prev_step {
        curr_step - prev_step
    } else {
        BARREL_N_STEPS - prev_step + curr_step
    };

    params.pending_spawns.clear();
    for i in 0..n {
        let step = (prev_step + 1 + i) % BARREL_N_STEPS;
        for ch in 0..BARREL_N_CHANNELS {
            if params.pattern[step][ch] {
                params.pending_spawns.push((step, ch));
            }
        }
    }
}

// ── Marble spawning from barrel triggers ──────────────────────────────────────
// Reads params.pending_spawns written by rotate_barrel_system (must run after it).

pub fn barrel_spawn_system(
    mut params: ResMut<BarrelParams>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chute_params: Res<ChuteParams>,
    vib_params: Res<VibraphoneParams>,
    marble_col: Res<MarbleCollisions>,
    mut all_runs: ResMut<RunHistory>,
    snare: Query<&GlobalTransform, With<SnareDrum>>,
) {
    if params.pending_spawns.is_empty() {
        return;
    }

    let snare_top_y = snare
        .single()
        .map(|gt| gt.translation().y + SNARE_HALF_HEIGHT)
        .unwrap_or(CHUTE_ORIGIN_Y);

    // Clone so we can iterate while mutating other parts of params (we only mutate pending later)
    let triggers: Vec<(usize, usize)> = params.pending_spawns.drain(..).collect();

    // Find steps that have both chute and drop active → pair them in one run for Δt analysis
    let chute_steps: Vec<usize> =
        triggers.iter().filter(|&&(_, ch)| ch == BARREL_CH_CHUTE).map(|&(s, _)| s).collect();
    let drop_steps: Vec<usize> =
        triggers.iter().filter(|&&(_, ch)| ch == BARREL_CH_DROP).map(|&(s, _)| s).collect();
    let paired_steps: Vec<usize> =
        chute_steps.iter().copied().filter(|s| drop_steps.contains(s)).collect();

    let mut fired_chute: Vec<usize> = Vec::new();
    let mut fired_drop: Vec<usize> = Vec::new();

    // Paired chute+drop → same run_idx for Δt analysis
    for &step in &paired_steps {
        let run_idx = all_runs.push_new_run();
        if let Some(run) = all_runs.get_run_mut(run_idx) {
            run.chute_exit = Some(chute_params.exit_pos);
        }
        let pos = jittered_spawn(snare_top_y);
        spawn_marble(&mut commands, &mut meshes, &mut materials, pos, marble_col.0, run_idx);
        spawn_chute_marble(
            &mut commands,
            &mut meshes,
            &mut materials,
            &chute_params,
            marble_col.0,
            run_idx,
        );
        fired_chute.push(step);
        fired_drop.push(step);
    }

    for &(step, ch) in &triggers {
        match ch {
            c if c == BARREL_CH_CHUTE && !fired_chute.contains(&step) => {
                let run_idx = all_runs.push_new_run();
                if let Some(run) = all_runs.get_run_mut(run_idx) {
                    run.chute_exit = Some(chute_params.exit_pos);
                }
                spawn_chute_marble(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &chute_params,
                    marble_col.0,
                    run_idx,
                );
            }
            c if c == BARREL_CH_DROP && !fired_drop.contains(&step) => {
                let run_idx = all_runs.push_new_run();
                let pos = jittered_spawn(snare_top_y);
                spawn_marble(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    pos,
                    marble_col.0,
                    run_idx,
                );
            }
            c if c >= BARREL_CH_VIB_FIRST => {
                let bar_idx = (c - BARREL_CH_VIB_FIRST) as u32;
                let run_idx = all_runs.push_new_run();
                if let Some(run) = all_runs.get_run_mut(run_idx) {
                    run.vib_bar_idx = Some(bar_idx);
                }
                spawn_vib_marble_for_bar(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &vib_params,
                    bar_idx,
                    marble_col.0,
                    run_idx,
                );
            }
            _ => {}
        }
    }
}

// ── Gizmos – active pegs and playhead indicator ───────────────────────────────

pub fn draw_barrel_gizmos(mut gizmos: Gizmos, params: Res<BarrelParams>) {
    if !params.show_pegs {
        return;
    }

    let step_angle = std::f32::consts::TAU / BARREL_N_STEPS as f32;
    let ch_width = BARREL_WIDTH / BARREL_N_CHANNELS as f32;

    for step in 0..BARREL_N_STEPS {
        for ch in 0..BARREL_N_CHANNELS {
            if !params.pattern[step][ch] {
                continue;
            }

            // Angular position of peg on cylinder surface relative to reader
            let alpha_rel = step as f32 * step_angle - params.angle;
            // Peg sits slightly proud of the cylinder surface
            let r = BARREL_RADIUS + 0.010;
            let y = BARREL_Y_POS + r * alpha_rel.cos();
            let z = BARREL_Z_POS + r * alpha_rel.sin();
            let x = (ch as f32 + 0.5) * ch_width - BARREL_WIDTH * 0.5;
            let pos = Vec3::new(x, y, z);

            let is_current = step == params.current_step;
            let color = if is_current {
                Color::srgb(1.00, 0.85, 0.10) // bright gold when at reader
            } else {
                let (r8, g8, b8) = channel_color_rgb(ch);
                Color::srgb(r8 as f32 / 255.0, g8 as f32 / 255.0, b8 as f32 / 255.0)
            };

            // Draw a small cross / plus at the peg position
            let s = if is_current { 0.014_f32 } else { 0.009_f32 };
            gizmos.line(pos - Vec3::X * s, pos + Vec3::X * s, color);
            gizmos.line(pos - Vec3::Y * s, pos + Vec3::Y * s, color);
            gizmos.line(pos - Vec3::Z * s, pos + Vec3::Z * s, color);
        }
    }

    // Draw a bright circle at the reader bar position showing which angle is "now"
    if params.enabled {
        let n = 64_usize;
        let mut prev = None::<Vec3>;
        for i in 0..=n {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            let pt = Vec3::new(
                BARREL_WIDTH * 0.5 + 0.04,
                BARREL_Y_POS + (BARREL_RADIUS + 0.020) * t.cos(),
                BARREL_Z_POS + (BARREL_RADIUS + 0.020) * t.sin(),
            );
            if let Some(p) = prev {
                gizmos.line(p, pt, Color::srgba(1.0, 0.5, 0.1, 0.5));
            }
            prev = Some(pt);
        }
    }
}

// ── Pattern editor UI ─────────────────────────────────────────────────────────

const CELL_W: f32 = 5.0; // narrower to fit 192 steps on screen
const CELL_H: f32 = 11.0;
const LABEL_W: f32 = 52.0;
const STEP_HEADER_H: f32 = 18.0;
const CHANNEL_GROUP_GAP: f32 = 3.0; // visual gap between chute/drop and vibs

pub fn barrel_editor_ui(mut contexts: EguiContexts, mut params: ResMut<BarrelParams>) {
    let ctx = contexts.ctx_mut().unwrap();

    // Small toggle button anchored top-left
    egui::Window::new("Barrel")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 8.0))
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            let label = if params.editor_open {
                "▼ Barrel Sequencer"
            } else {
                "► Barrel Sequencer"
            };
            if ui.small_button(label).clicked() {
                params.editor_open = !params.editor_open;
            }
        });

    if !params.editor_open {
        return;
    }

    egui::Window::new("Barrel Sequencer##editor")
        .default_pos([10.0, 40.0])
        .default_size([820.0, 560.0])
        .resizable(true)
        .title_bar(false)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new(egui::RichText::new("Barrel Sequencer").strong())
                .id_salt("barrel_editor_header")
                .default_open(true)
                .show(ui, |ui| {
                    draw_transport(ui, &mut *params);
                    ui.separator();
                    draw_pattern_grid(ui, &mut *params);
                });
        });
}

fn draw_transport(ui: &mut egui::Ui, params: &mut BarrelParams) {
    ui.horizontal(|ui| {
        // Play / Stop button
        let (btn_label, btn_color) = if params.enabled {
            ("■ Stop", egui::Color32::from_rgb(220, 80, 60))
        } else {
            ("▶ Play", egui::Color32::from_rgb(60, 180, 80))
        };
        if ui
            .add(
                egui::Button::new(egui::RichText::new(btn_label).color(btn_color).strong())
                    .min_size(egui::vec2(70.0, 24.0)),
            )
            .clicked()
        {
            params.enabled = !params.enabled;
            if params.enabled {
                // Restart from just before step 0
                params.reset_position();
            }
        }

        // BPM (derived from RPM × N_STEPS)
        let mut bpm = params.rpm * BARREL_N_STEPS as f32;
        ui.label("BPM:");
        if ui
            .add(egui::DragValue::new(&mut bpm).speed(0.5).range(10.0..=600.0))
            .changed()
        {
            params.rpm = bpm / BARREL_N_STEPS as f32;
        }
        ui.monospace(format!("({:.3} RPM)", params.rpm));

        ui.separator();
        ui.checkbox(&mut params.show_pegs, "Show 3D pegs");

        ui.separator();
        if ui.small_button("Reset pos").clicked() {
            params.reset_position();
        }
        if ui.small_button("Clear all").clicked() {
            params.clear_pattern();
        }
    });
}

fn draw_pattern_grid(ui: &mut egui::Ui, params: &mut BarrelParams) {
    // Step-number header row
    let total_grid_w = LABEL_W + BARREL_N_STEPS as f32 * CELL_W;
    let total_grid_h = STEP_HEADER_H
        + BARREL_N_CHANNELS as f32 * CELL_H
        + CHANNEL_GROUP_GAP; // gap after row 1

    egui::ScrollArea::both()
        .id_salt("barrel_grid_scroll")
        .max_width(ui.available_width())
        .max_height(ui.available_height().min(520.0))
        .show(ui, |ui| {
            let (outer_rect, _) = ui.allocate_exact_size(
                egui::vec2(total_grid_w, total_grid_h),
                egui::Sense::hover(),
            );
            let painter = ui.painter_at(outer_rect);

            // ── Step header ───────────────────────────────────────────────
            // Shows beat numbers (1-16) with tick marks for 8th and triplet positions.
            let header_top = outer_rect.min.y;
            for step in 0..BARREL_N_STEPS {
                let x = outer_rect.min.x + LABEL_W + step as f32 * CELL_W;
                if step % BARREL_STEPS_PER_BEAT == 0 {
                    // Beat number label
                    let beat = step / BARREL_STEPS_PER_BEAT + 1;
                    let cx = x + CELL_W * (BARREL_STEPS_PER_BEAT as f32 * 0.5);
                    painter.text(
                        egui::pos2(cx, header_top + STEP_HEADER_H * 0.4),
                        egui::Align2::CENTER_CENTER,
                        format!("{beat}"),
                        egui::FontId::monospace(8.0),
                        egui::Color32::from_rgb(180, 180, 190),
                    );
                    // Full-height tick for beat
                    painter.line_segment(
                        [
                            egui::pos2(x, header_top + STEP_HEADER_H - 5.0),
                            egui::pos2(x, header_top + STEP_HEADER_H),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 120, 135)),
                    );
                } else if step % 6 == 0 {
                    // 8th-note tick (÷2 within beat)
                    painter.line_segment(
                        [
                            egui::pos2(x, header_top + STEP_HEADER_H - 3.0),
                            egui::pos2(x, header_top + STEP_HEADER_H),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 100)),
                    );
                } else if step % 4 == 0 {
                    // Triplet tick (÷3 within beat)
                    painter.line_segment(
                        [
                            egui::pos2(x, header_top + STEP_HEADER_H - 2.0),
                            egui::pos2(x, header_top + STEP_HEADER_H),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 70, 120)),
                    );
                }
            }

            // ── Channel rows ──────────────────────────────────────────────
            let grid_top = outer_rect.min.y + STEP_HEADER_H;

            // Interaction region covering the cells (not the labels)
            let cells_rect = egui::Rect::from_min_size(
                egui::pos2(outer_rect.min.x + LABEL_W, grid_top),
                egui::vec2(
                    BARREL_N_STEPS as f32 * CELL_W,
                    BARREL_N_CHANNELS as f32 * CELL_H + CHANNEL_GROUP_GAP,
                ),
            );
            let interact_resp =
                ui.allocate_rect(cells_rect, egui::Sense::click_and_drag());

            // Handle click / drag interactions
            let ptr_pos = interact_resp.interact_pointer_pos();
            let left_down = ui.input(|i| i.pointer.primary_down());
            let right_down = ui.input(|i| i.pointer.secondary_down());

            if interact_resp.drag_started() {
                // Determine paint value from the first cell under the pointer
                if let Some(pos) = ptr_pos {
                    if let Some((step, ch, row_y)) = cell_at(pos, outer_rect.min, grid_top) {
                        let _ = row_y;
                        if left_down {
                            let current = params.pattern[step][ch];
                            params.drag_paint_val = Some(!current);
                            params.pattern[step][ch] = !current;
                        } else if right_down {
                            params.drag_paint_val = Some(false);
                            params.pattern[step][ch] = false;
                        }
                    }
                }
            } else if interact_resp.dragged() {
                if let (Some(pos), Some(paint_val)) = (ptr_pos, params.drag_paint_val) {
                    if let Some((step, ch, _)) = cell_at(pos, outer_rect.min, grid_top) {
                        params.pattern[step][ch] = paint_val;
                    }
                }
            } else if interact_resp.drag_stopped() {
                params.drag_paint_val = None;
            } else if interact_resp.clicked() {
                if let Some(pos) = ptr_pos {
                    if let Some((step, ch, _)) = cell_at(pos, outer_rect.min, grid_top) {
                        params.pattern[step][ch] ^= true;
                    }
                }
            }

            // ── Draw cells ────────────────────────────────────────────────
            let current_step = params.current_step;

            for ch in 0..BARREL_N_CHANNELS {
                // Visual gap after the snare-drop row (row index 1)
                let y_gap = if ch >= 2 { CHANNEL_GROUP_GAP } else { 0.0 };
                let row_y = grid_top + y_gap + ch as f32 * CELL_H;

                // Row label
                let (lr, lg, lb) = channel_color_rgb(ch);
                let label_color =
                    egui::Color32::from_rgb(lr, lg, lb);
                painter.text(
                    egui::pos2(outer_rect.min.x + LABEL_W - 4.0, row_y + CELL_H * 0.5),
                    egui::Align2::RIGHT_CENTER,
                    channel_name(ch),
                    egui::FontId::monospace(9.0),
                    label_color,
                );

                // Cells
                for step in 0..BARREL_N_STEPS {
                    let cell_x = outer_rect.min.x + LABEL_W + step as f32 * CELL_W;
                    let cell_rect = egui::Rect::from_min_size(
                        egui::pos2(cell_x + 0.5, row_y + 0.5),
                        egui::vec2(CELL_W - 1.0, CELL_H - 1.0),
                    );

                    let is_active = params.pattern[step][ch];
                    let is_current = step == current_step && params.enabled;
                    // Subdivision positions within a beat (beat = BARREL_STEPS_PER_BEAT steps)
                    let is_beat_start  = step % BARREL_STEPS_PER_BEAT == 0;
                    let is_eighth      = step % 6 == 0 && !is_beat_start;
                    let is_triplet     = step % 4 == 0 && step % 6 != 0 && !is_beat_start;

                    let color = match (is_active, is_current) {
                        (true, true) => egui::Color32::from_rgb(255, 220, 30), // active + at reader
                        (true, false) => {
                            let (r8, g8, b8) = channel_color_rgb(ch);
                            egui::Color32::from_rgb(r8, g8, b8)
                        }
                        (false, true) => egui::Color32::from_rgb(80, 65, 20), // cursor bg
                        (false, false) => {
                            if is_beat_start {
                                egui::Color32::from_rgb(42, 40, 55) // beat: slightly bright
                            } else if is_eighth {
                                egui::Color32::from_rgb(30, 30, 44) // 8th note position
                            } else if is_triplet {
                                egui::Color32::from_rgb(27, 22, 40) // triplet: slight purple tint
                            } else {
                                egui::Color32::from_rgb(18, 18, 26) // other subdivisions
                            }
                        }
                    };

                    painter.rect_filled(cell_rect, 1.0, color);

                    // Highlight current-step column border
                    if is_current {
                        painter.rect_stroke(
                            cell_rect,
                            1.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 200, 0)),
                            egui::StrokeKind::Outside,
                        );
                    }
                }
            }

            // Grid lines — three tiers matching the visual subdivisions in the header
            let grid_bottom = grid_top + BARREL_N_CHANNELS as f32 * CELL_H + CHANNEL_GROUP_GAP;
            for step in 0..=BARREL_N_STEPS {
                let x = outer_rect.min.x + LABEL_W + step as f32 * CELL_W;
                let (stroke_w, color) = if step % BARREL_STEPS_PER_BEAT == 0 {
                    // Beat line — solid, most visible
                    (1.0, egui::Color32::from_rgba_premultiplied(110, 110, 140, 140))
                } else if step % 6 == 0 {
                    // 8th-note line (÷2 per beat)
                    (0.5, egui::Color32::from_rgba_premultiplied(60, 60, 90, 80))
                } else if step % 4 == 0 {
                    // Triplet line (÷3 per beat) — purple tint
                    (0.5, egui::Color32::from_rgba_premultiplied(80, 50, 110, 70))
                } else {
                    continue;
                };
                painter.line_segment(
                    [egui::pos2(x, grid_top), egui::pos2(x, grid_bottom)],
                    egui::Stroke::new(stroke_w, color),
                );
            }
        });

    // Row controls below the grid (clear / fill per channel group)
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Row ops:");
        if ui.small_button("Clear chute row").clicked() {
            for s in &mut params.pattern {
                s[0] = false;
            }
        }
        if ui.small_button("Clear drop row").clicked() {
            for s in &mut params.pattern {
                s[1] = false;
            }
        }
        if ui.small_button("Clear all vib rows").clicked() {
            for s in &mut params.pattern {
                for ch in 2..BARREL_N_CHANNELS {
                    s[ch] = false;
                }
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("Quick fill:");
        quick_fill_buttons(ui, &mut params.pattern);
    });
}

/// Convert pointer position to (step, channel, row_y) within the grid, or None.
fn cell_at(
    pos: egui::Pos2,
    outer_min: egui::Pos2,
    grid_top: f32,
) -> Option<(usize, usize, f32)> {
    let rel_x = pos.x - (outer_min.x + LABEL_W);
    let rel_y = pos.y - grid_top;

    if rel_x < 0.0 || rel_y < 0.0 {
        return None;
    }

    // Account for the visual gap inserted after channel 1
    let adj_y = if rel_y >= 2.0 * CELL_H + CHANNEL_GROUP_GAP {
        rel_y - CHANNEL_GROUP_GAP
    } else {
        rel_y
    };

    let step = (rel_x / CELL_W) as usize;
    let ch = (adj_y / CELL_H) as usize;

    if step < BARREL_N_STEPS && ch < BARREL_N_CHANNELS {
        Some((step, ch, grid_top + ch as f32 * CELL_H))
    } else {
        None
    }
}

fn quick_fill_buttons(ui: &mut egui::Ui, pattern: &mut Vec<Vec<bool>>) {
    // Quarter notes: once per beat (every BARREL_STEPS_PER_BEAT steps)
    if ui.small_button("Chute quarter").clicked() {
        for step in 0..BARREL_N_STEPS {
            pattern[step][0] = step % BARREL_STEPS_PER_BEAT == 0;
        }
    }
    // Eighth notes: twice per beat (every 6 steps)
    if ui.small_button("Drop 8ths").clicked() {
        for step in 0..BARREL_N_STEPS {
            pattern[step][1] = step % 6 == 0;
        }
    }
    // Triplets: three per beat (every 4 steps)
    if ui.small_button("Chute triplets").clicked() {
        for step in 0..BARREL_N_STEPS {
            pattern[step][0] = step % 4 == 0;
        }
    }
    // C major arpeggio on vibraphone bars 0,4,7,12 (C, E, G, C)
    // Steps mapped to same fractional positions as before (every 2 beats = every 24 steps)
    if ui.small_button("Vib C-major arp").clicked() {
        let vib_steps = [0usize, 24, 48, 72, 96, 120, 144, 168];
        let vib_bars = [2usize, 6, 9, 14]; // BARREL_CH_VIB_FIRST+offset: bars 0,4,7,12
        for step in 0..BARREL_N_STEPS {
            for ch in 2..BARREL_N_CHANNELS {
                pattern[step][ch] = false;
            }
        }
        for (i, &step) in vib_steps.iter().enumerate() {
            let ch = vib_bars[i % vib_bars.len()];
            if ch < BARREL_N_CHANNELS {
                pattern[step][ch] = true;
            }
        }
    }
}
