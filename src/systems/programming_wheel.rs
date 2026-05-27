use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rand::RngExt;

use crate::components::instrument::{Instrument, MarbleSpawner};
use crate::components::programming_wheel::{spawn_programming_wheel, ProgrammingWheelCylinder};
use crate::components::snare::SnareDrum;
use crate::resources::chute_params::ChuteParams;
use crate::resources::snare_params::SnareParams;
use crate::resources::constants::*;
use crate::resources::marble_collisions::MarbleCollisions;
use crate::resources::marble_runs::RunHistory;
use crate::resources::programming_wheel_params::{
    channel_color_rgb, channel_jitter_xz, channel_name, channel_target,
    snap_beat, snap_label,
    ChannelTarget, DragState, ProgrammingWheelParams, WheelNote,
    WHEEL_CH_CHUTE, WHEEL_CH_DROP, WHEEL_CH_VIB_FIRST,
};
use crate::systems::marble::{chute_spawn_pos, spawn_marble};

// ── Setup ─────────────────────────────────────────────────────────────────────

pub fn setup_programming_wheel_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_programming_wheel(&mut commands, &mut meshes, &mut materials);
}

/// Spawn one `MarbleSpawner` entity per non-vibraphone channel (0 = ghost snare,
/// 1–7 = snare variants).  Vibraphone bar spawners are created by
/// `spawn_vibraphone` and tagged `VibraphoneEntity` so they rebuild with the
/// instrument.  All spawner positions are set to their correct world locations
/// by `sync_instrument_spawners` on the very first Update frame.
pub fn setup_spawners_system(mut commands: Commands) {
    for ch in 0..WHEEL_CH_VIB_FIRST {
        commands.spawn((Transform::default(), MarbleSpawner { channel: ch }));
    }
}

// ── Spawner synchronisation ───────────────────────────────────────────────────

/// Runs every Update frame (before the spawn system) and repositions every
/// `MarbleSpawner` entity to track its instrument's current world location.
///
/// - **Ghost snare (ch 0)**: positioned at the chute entry from `ChuteParams`.
/// - **Snare variants (ch 1–7)**: above the snare drum with their X offset.
/// - **Vibraphone bars (ch 8–44)**: 1 m above each bar's current world centre.
///
/// Because the position is derived from entity `GlobalTransform`s, moving any
/// instrument entity (or rebuilding it with new params) automatically shifts
/// the corresponding spawn point on the next frame.
pub fn sync_instrument_spawners(
    instruments: Query<(&Instrument, &GlobalTransform)>,
    chute_params: Res<ChuteParams>,
    snare_params: Res<SnareParams>,
    snare_q: Query<&GlobalTransform, With<SnareDrum>>,
    mut spawners: Query<(&MarbleSpawner, &mut Transform)>,
) {
    let snare_world = snare_q
        .single()
        .map(|gt| gt.translation())
        .unwrap_or_default();

    for (spawner, mut tf) in &mut spawners {
        tf.translation = match channel_target(spawner.channel) {
            ChannelTarget::GhostSnare => chute_spawn_pos(&chute_params, snare_params.pos),

            ChannelTarget::Snare { x_offset } => Vec3::new(
                snare_world.x + x_offset,
                snare_world.y + SNARE_HALF_HEIGHT + SPAWN_HEIGHT,
                snare_world.z,
            ),

            ChannelTarget::VibBar { bar_idx } => {
                let vib_ch = WHEEL_CH_VIB_FIRST + bar_idx as usize;
                instruments
                    .iter()
                    .find(|(instr, _)| instr.channel == vib_ch)
                    .map(|(_, gt)| {
                        // gt.translation() is the bar's world-space centre.
                        // Add half-thickness (bar top face) + spawn height.
                        let p = gt.translation();
                        Vec3::new(
                            p.x,
                            p.y + VIB_BAR_THICKNESS * 0.5 + VIB_SPAWN_HEIGHT,
                            p.z,
                        )
                    })
                    .unwrap_or_default()
            }
        };
    }
}

// ── Rotation + beat-crossing detection ───────────────────────────────────────

pub fn rotate_programming_wheel_system(
    time: Res<Time>,
    mut params: ResMut<ProgrammingWheelParams>,
    mut cylinder_q: Query<&mut Transform, With<ProgrammingWheelCylinder>>,
) {
    if let Ok(mut tf) = cylinder_q.single_mut() {
        let base = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let spin = Quat::from_rotation_y(-params.angle);
        tf.rotation = base * spin;
    }

    if !params.enabled {
        return;
    }

    let prev_angle = params.angle;
    let delta = std::f32::consts::TAU * (params.rpm / 60.0) * time.delta_secs();
    params.angle = (params.angle + delta).rem_euclid(std::f32::consts::TAU);

    // Convert angles to beats for musically-natural comparison
    let bpr = PROGRAMMING_WHEEL_BEATS_PER_REV;
    let prev_beat = prev_angle / std::f32::consts::TAU * bpr;
    let curr_beat = params.angle / std::f32::consts::TAU * bpr;
    params.current_beat = curr_beat;

    let fired: Vec<usize> = params
        .notes
        .iter()
        .filter_map(|note| {
            let nb = note.beat.rem_euclid(bpr);
            let crossed = if curr_beat >= prev_beat {
                nb > prev_beat && nb <= curr_beat
            } else {
                nb > prev_beat || nb <= curr_beat
            };
            if crossed { Some(note.channel) } else { None }
        })
        .collect();
    params.pending_spawns.clear();
    params.pending_spawns.extend(fired);
}

// ── Marble spawning ───────────────────────────────────────────────────────────

/// Spawns marbles for every channel queued in `ProgrammingWheelParams::pending_spawns`.
///
/// Spawn positions come exclusively from `MarbleSpawner` entities, which are
/// kept in sync with instrument world positions by `sync_instrument_spawners`.
/// Per-channel XZ jitter (if any) is read from `channel_jitter_xz` and applied
/// on top of the spawner's base position.
pub fn programming_wheel_spawn_system(
    mut params: ResMut<ProgrammingWheelParams>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    marble_col: Res<MarbleCollisions>,
    mut all_runs: ResMut<RunHistory>,
    spawners: Query<(&MarbleSpawner, &Transform)>,
) {
    if params.pending_spawns.is_empty() {
        return;
    }

    let channels: Vec<usize> = params.pending_spawns.drain(..).collect();

    for ch in channels {
        let Some((_, spawner_tf)) = spawners.iter().find(|(s, _)| s.channel == ch) else {
            continue; // spawner entity not yet created (shouldn't happen after setup)
        };
        let base = spawner_tf.translation;
        let jitter = channel_jitter_xz(ch);
        let pos = if jitter > 0.0 {
            let mut rng = rand::rng();
            Vec3::new(
                base.x + rng.random_range(-jitter..jitter),
                base.y,
                base.z + rng.random_range(-jitter..jitter),
            )
        } else {
            base
        };
        let run_idx = all_runs.push_new_run(ch);
        spawn_marble(
            &mut commands,
            &mut meshes,
            &mut materials,
            pos,
            marble_col.0,
            run_idx,
            ch,
        );
    }
}

// ── Gizmos ────────────────────────────────────────────────────────────────────

pub fn draw_programming_wheel_gizmos(
    mut gizmos: Gizmos,
    params: Res<ProgrammingWheelParams>,
) {
    if !params.show_pegs {
        return;
    }

    let bpr = PROGRAMMING_WHEEL_BEATS_PER_REV;
    let ch_width = PROGRAMMING_WHEEL_WIDTH / PROGRAMMING_WHEEL_N_CHANNELS as f32;

    for note in &params.notes {
        let note_angle = note.beat / bpr * std::f32::consts::TAU;
        let alpha_rel = note_angle - params.angle;
        let r = PROGRAMMING_WHEEL_RADIUS + 0.010;
        let y = PROGRAMMING_WHEEL_Y_POS + r * alpha_rel.cos();
        let z = PROGRAMMING_WHEEL_Z_POS + r * alpha_rel.sin();
        let x = (note.channel as f32 + 0.5) * ch_width - PROGRAMMING_WHEEL_WIDTH * 0.5;
        let pos = Vec3::new(x, y, z);

        let at_reader = (alpha_rel.rem_euclid(std::f32::consts::TAU) < 0.05)
            || (alpha_rel.rem_euclid(std::f32::consts::TAU) > std::f32::consts::TAU - 0.05);
        let color = if at_reader {
            Color::srgb(1.00, 0.85, 0.10)
        } else {
            let (r8, g8, b8) = channel_color_rgb(note.channel);
            Color::srgb(r8 as f32 / 255.0, g8 as f32 / 255.0, b8 as f32 / 255.0)
        };

        let s = if at_reader { 0.014_f32 } else { 0.009_f32 };
        gizmos.line(pos - Vec3::X * s, pos + Vec3::X * s, color);
        gizmos.line(pos - Vec3::Y * s, pos + Vec3::Y * s, color);
        gizmos.line(pos - Vec3::Z * s, pos + Vec3::Z * s, color);
    }

    if params.enabled {
        let n = 64_usize;
        let mut prev = None::<Vec3>;
        for i in 0..=n {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            let pt = Vec3::new(
                PROGRAMMING_WHEEL_WIDTH * 0.5 + 0.04,
                PROGRAMMING_WHEEL_Y_POS + (PROGRAMMING_WHEEL_RADIUS + 0.020) * t.cos(),
                PROGRAMMING_WHEEL_Z_POS + (PROGRAMMING_WHEEL_RADIUS + 0.020) * t.sin(),
            );
            if let Some(p) = prev {
                gizmos.line(p, pt, Color::srgba(1.0, 0.5, 0.1, 0.5));
            }
            prev = Some(pt);
        }
    }
}

// ── Piano-roll editor UI ──────────────────────────────────────────────────────

const CELL_H: f32 = 11.0;
const LABEL_W: f32 = 52.0;
const BEAT_HEADER_H: f32 = 24.0;
const CHANNEL_GROUP_GAP: f32 = 3.0;
const RESIZE_HANDLE_PX: f32 = 5.0;

pub fn programming_wheel_editor_ui(
    mut contexts: EguiContexts,
    mut params: ResMut<ProgrammingWheelParams>,
) {
    let ctx = contexts.ctx_mut().unwrap();

    let default_w = LABEL_W + PROGRAMMING_WHEEL_BEATS_PER_REV * params.px_per_beat + 24.0;
    let default_h = BEAT_HEADER_H
        + PROGRAMMING_WHEEL_N_CHANNELS as f32 * CELL_H
        + CHANNEL_GROUP_GAP
        + 34.0 + 80.0 + 24.0;

    egui::Window::new("Programming Wheel")
        .default_pos([8.0, 8.0])
        .default_size([default_w, default_h])
        .resizable(true)
        .title_bar(false)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new(egui::RichText::new("Programming Wheel").strong())
                .id_salt("programming_wheel_editor_header")
                .default_open(true)
                .show(ui, |ui| {
                    draw_transport(ui, &mut *params);
                    ui.separator();
                    draw_piano_roll(ui, &mut *params);
                });
        });
}

fn draw_transport(ui: &mut egui::Ui, params: &mut ProgrammingWheelParams) {
    ui.horizontal(|ui| {
        let (btn_label, btn_color) = if params.enabled {
            ("■ Stop", egui::Color32::from_rgb(220, 80, 60))
        } else {
            ("▶ Play", egui::Color32::from_rgb(60, 180, 80))
        };
        if ui
            .add(egui::Button::new(
                egui::RichText::new(btn_label).color(btn_color).strong()
            ).min_size(egui::vec2(70.0, 24.0)))
            .clicked()
        {
            params.enabled = !params.enabled;
            if params.enabled {
                params.reset_position();
            }
        }

        let bpr = PROGRAMMING_WHEEL_BEATS_PER_REV;
        let mut bpm = params.rpm * bpr;
        ui.label("BPM:");
        if ui.add(egui::DragValue::new(&mut bpm).speed(0.5).range(10.0..=600.0)).changed() {
            params.rpm = bpm / bpr;
        }
        ui.monospace(format!("({:.4} RPM)", params.rpm));

        ui.separator();
        ui.label("Snap:");
        egui::ComboBox::from_id_salt("snap_select")
            .selected_text(snap_label(params.snap_beats))
            .width(55.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut params.snap_beats, 0.0,       "Free");
                ui.selectable_value(&mut params.snap_beats, 0.25,      "1/16");
                ui.selectable_value(&mut params.snap_beats, 1.0/3.0,   "1/8T");
                ui.selectable_value(&mut params.snap_beats, 0.5,       "1/8");
                ui.selectable_value(&mut params.snap_beats, 1.0,       "1/4");
            });

        ui.label("Zoom:");
        ui.add(egui::Slider::new(&mut params.px_per_beat, 5.0..=60.0).show_value(false));

        ui.separator();
        ui.checkbox(&mut params.show_pegs, "3D pegs");
        ui.separator();
        if ui.small_button("Reset pos").clicked() { params.reset_position(); }
        if ui.small_button("Clear all").clicked()  { params.clear_notes(); }
    });
}

/// Returns the top-Y of the row for `ch` inside the grid.
/// A visual gap is inserted between the snare section (ch 0–7) and the
/// vibraphone section (ch 8+) to make the two groups easy to distinguish.
fn channel_row_y(ch: usize, grid_top: f32) -> f32 {
    let gap = if ch >= WHEEL_CH_VIB_FIRST { CHANNEL_GROUP_GAP } else { 0.0 };
    grid_top + gap + ch as f32 * CELL_H
}

fn y_to_channel(y: f32, grid_top: f32) -> usize {
    let rel = y - grid_top;
    let vib_boundary = WHEEL_CH_VIB_FIRST as f32 * CELL_H;
    let adj = if rel >= vib_boundary + CHANNEL_GROUP_GAP {
        rel - CHANNEL_GROUP_GAP
    } else {
        rel
    };
    ((adj / CELL_H) as usize).min(PROGRAMMING_WHEEL_N_CHANNELS - 1)
}

fn note_screen_rect(
    note: &WheelNote,
    grid_left: f32,
    grid_top: f32,
    px_per_beat: f32,
) -> egui::Rect {
    let x = grid_left + note.beat * px_per_beat;
    let w = (note.length * px_per_beat).max(2.0);
    let y = channel_row_y(note.channel, grid_top);
    egui::Rect::from_min_size(egui::pos2(x, y + 0.5), egui::vec2(w, CELL_H - 1.0))
}

enum NoteHit { Body, RightEdge }

fn find_note_at(
    pos: egui::Pos2,
    notes: &[WheelNote],
    grid_left: f32,
    grid_top: f32,
    px_per_beat: f32,
) -> Option<(usize, NoteHit)> {
    for (i, note) in notes.iter().enumerate().rev() {
        let r = note_screen_rect(note, grid_left, grid_top, px_per_beat);
        if r.contains(pos) {
            let hit = if pos.x >= r.max.x - RESIZE_HANDLE_PX {
                NoteHit::RightEdge
            } else {
                NoteHit::Body
            };
            return Some((i, hit));
        }
    }
    None
}

fn draw_piano_roll(ui: &mut egui::Ui, params: &mut ProgrammingWheelParams) {
    let px = params.px_per_beat;
    let bpr = PROGRAMMING_WHEEL_BEATS_PER_REV;
    let n_ch = PROGRAMMING_WHEEL_N_CHANNELS;

    let total_w = LABEL_W + bpr * px;
    let total_h = BEAT_HEADER_H + n_ch as f32 * CELL_H + CHANNEL_GROUP_GAP;

    egui::ScrollArea::both()
        .id_salt("programming_wheel_roll_scroll")
        .max_width(ui.available_width())
        .max_height(ui.available_height().min(500.0))
        .show(ui, |ui| {
            let (outer_rect, _) = ui.allocate_exact_size(
                egui::vec2(total_w, total_h),
                egui::Sense::hover(),
            );
            let painter = ui.painter_at(outer_rect);
            let grid_left = outer_rect.min.x + LABEL_W;
            let grid_top  = outer_rect.min.y + BEAT_HEADER_H;

            // ── Beat / bar header ─────────────────────────────────────────
            let hdr_top = outer_rect.min.y;
            let n_beats = bpr as usize;
            // Bar numbers centred over each 4-beat span
            for bar in 0..n_beats / 4 {
                let bar_x = grid_left + bar as f32 * 4.0 * px;
                let cx    = bar_x + 2.0 * px;
                painter.text(
                    egui::pos2(cx, hdr_top + 7.0),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", bar + 1),
                    egui::FontId::proportional(10.0),
                    egui::Color32::from_rgb(200, 200, 215),
                );
                painter.line_segment(
                    [egui::pos2(bar_x, hdr_top + 14.0), egui::pos2(bar_x, hdr_top + BEAT_HEADER_H)],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(130, 130, 155)),
                );
            }
            // Beat ticks within bars
            for b in 0..n_beats {
                let x = grid_left + b as f32 * px;
                if b % 4 != 0 {
                    let beat_in_bar = b % 4 + 1;
                    painter.text(
                        egui::pos2(x + 2.0, hdr_top + BEAT_HEADER_H - 3.0),
                        egui::Align2::LEFT_BOTTOM,
                        format!("{beat_in_bar}"),
                        egui::FontId::monospace(7.0),
                        egui::Color32::from_rgb(120, 120, 140),
                    );
                    painter.line_segment(
                        [egui::pos2(x, hdr_top + 16.0), egui::pos2(x, hdr_top + BEAT_HEADER_H)],
                        egui::Stroke::new(0.8, egui::Color32::from_rgb(90, 90, 115)),
                    );
                }
                // 8th-note marks
                let xh = grid_left + (b as f32 + 0.5) * px;
                painter.line_segment(
                    [egui::pos2(xh, hdr_top + 20.0), egui::pos2(xh, hdr_top + BEAT_HEADER_H)],
                    egui::Stroke::new(0.5, egui::Color32::from_rgb(60, 60, 80)),
                );
            }

            // ── Channel row backgrounds and labels ────────────────────────
            for ch in 0..n_ch {
                let row_y = channel_row_y(ch, grid_top);
                let (lr, lg, lb) = channel_color_rgb(ch);
                // Alternating row bg
                let row_bg = if ch % 2 == 0 {
                    egui::Color32::from_rgb(22, 22, 30)
                } else {
                    egui::Color32::from_rgb(18, 18, 25)
                };
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(grid_left, row_y),
                        egui::vec2(bpr * px, CELL_H),
                    ),
                    0.0, row_bg,
                );
                painter.text(
                    egui::pos2(outer_rect.min.x + LABEL_W - 4.0, row_y + CELL_H * 0.5),
                    egui::Align2::RIGHT_CENTER,
                    channel_name(ch),
                    egui::FontId::monospace(9.0),
                    egui::Color32::from_rgb(lr, lg, lb),
                );
            }

            // ── Beat grid lines ───────────────────────────────────────────
            let grid_bottom = grid_top + n_ch as f32 * CELL_H + CHANNEL_GROUP_GAP;
            for b in 0..=n_beats {
                let x = grid_left + b as f32 * px;
                let (sw, col) = if b % 4 == 0 {
                    (1.0, egui::Color32::from_rgba_premultiplied(110, 110, 140, 120))
                } else {
                    (0.5, egui::Color32::from_rgba_premultiplied(55, 55, 80, 80))
                };
                painter.line_segment(
                    [egui::pos2(x, grid_top), egui::pos2(x, grid_bottom)],
                    egui::Stroke::new(sw, col),
                );
            }
            // 8th-note grid lines (every 0.5 beats)
            for b2 in 0..n_beats * 2 {
                if b2 % 2 != 0 {
                    let x = grid_left + b2 as f32 * px * 0.5;
                    painter.line_segment(
                        [egui::pos2(x, grid_top), egui::pos2(x, grid_bottom)],
                        egui::Stroke::new(0.5, egui::Color32::from_rgba_premultiplied(40, 40, 65, 60)),
                    );
                }
            }

            // ── Interaction area ──────────────────────────────────────────
            let cells_rect = egui::Rect::from_min_size(
                egui::pos2(grid_left, grid_top),
                egui::vec2(bpr * px, n_ch as f32 * CELL_H + CHANNEL_GROUP_GAP),
            );
            let resp = ui.allocate_rect(cells_rect, egui::Sense::click_and_drag());
            let ptr  = resp.interact_pointer_pos();
            let snap = params.snap_beats;

            // Right-click: delete note
            if resp.secondary_clicked() {
                if let Some(pos) = ptr {
                    if let Some((idx, _)) = find_note_at(pos, &params.notes, grid_left, grid_top, px) {
                        params.notes.remove(idx);
                        params.drag_state = DragState::None;
                    }
                }
            }

            // Left drag start
            if resp.drag_started() && ui.input(|i| i.pointer.primary_down()) {
                if let Some(pos) = ptr {
                    let raw_beat = (pos.x - grid_left) / px;
                    let beat = snap_beat(raw_beat, snap).clamp(0.0, bpr - 0.001);
                    let ch = y_to_channel(pos.y, grid_top);
                    match find_note_at(pos, &params.notes, grid_left, grid_top, px) {
                        Some((idx, NoteHit::RightEdge)) => {
                            params.drag_state = DragState::Resizing { note_idx: idx };
                        }
                        Some((idx, NoteHit::Body)) => {
                            let offset = raw_beat - params.notes[idx].beat;
                            params.drag_state = DragState::Moving { note_idx: idx, beat_offset: offset };
                        }
                        None => {
                            let default_len = if snap > 0.0 { snap } else { 0.25 };
                            params.drag_state = DragState::Creating {
                                channel: ch,
                                start_beat: beat,
                                end_beat: beat + default_len,
                            };
                        }
                    }
                }
            }

            // Left drag ongoing
            if resp.dragged() {
                if let Some(pos) = ptr {
                    let raw_beat = (pos.x - grid_left) / px;
                    let beat = snap_beat(raw_beat, snap).clamp(0.0, bpr);
                    match &mut params.drag_state {
                        DragState::Creating { end_beat, .. } => {
                            *end_beat = beat;
                        }
                        DragState::Moving { note_idx, beat_offset } => {
                            let idx = *note_idx;
                            let offset = *beat_offset;
                            let new_beat = snap_beat(raw_beat - offset, snap).rem_euclid(bpr);
                            params.notes[idx].beat = new_beat;
                        }
                        DragState::Resizing { note_idx } => {
                            let idx = *note_idx;
                            let start = params.notes[idx].beat;
                            let new_len = (beat - start).max(0.01);
                            params.notes[idx].length = new_len;
                        }
                        DragState::None => {}
                    }
                }
            }

            // Left drag released
            if resp.drag_stopped() {
                if let DragState::Creating { channel, start_beat, end_beat } = params.drag_state {
                    let len = (end_beat - start_beat).abs().max(0.01);
                    let beat = start_beat.min(end_beat);
                    params.notes.push(WheelNote::new(channel, beat, len));
                }
                params.drag_state = DragState::None;
            }

            // Single click on empty: create note with default length
            if resp.clicked() && ui.input(|i| i.pointer.primary_pressed()) {
                if let Some(pos) = ptr {
                    if find_note_at(pos, &params.notes, grid_left, grid_top, px).is_none() {
                        let raw_beat = (pos.x - grid_left) / px;
                        let beat = snap_beat(raw_beat, snap).clamp(0.0, bpr - 0.001);
                        let ch = y_to_channel(pos.y, grid_top);
                        let default_len = if snap > 0.0 { snap } else { 0.25 };
                        params.notes.push(WheelNote::new(ch, beat, default_len));
                    }
                }
            }

            // ── Draw notes ────────────────────────────────────────────────
            // Preview note being created
            if let DragState::Creating { channel, start_beat, end_beat } = params.drag_state {
                let preview_beat = start_beat.min(end_beat);
                let preview_len  = (end_beat - start_beat).abs().max(0.01);
                let preview = WheelNote::new(channel, preview_beat, preview_len);
                let r = note_screen_rect(&preview, grid_left, grid_top, px);
                let (cr, cg, cb) = channel_color_rgb(channel);
                painter.rect_filled(r, 2.0, egui::Color32::from_rgba_premultiplied(cr, cg, cb, 140));
                painter.rect_stroke(r, 2.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(cr, cg, cb)), egui::StrokeKind::Outside);
            }

            for (i, note) in params.notes.iter().enumerate() {
                let r = note_screen_rect(note, grid_left, grid_top, px);
                let (cr, cg, cb) = channel_color_rgb(note.channel);

                let is_moving   = matches!(&params.drag_state, DragState::Moving   { note_idx, .. } if *note_idx == i);
                let is_resizing = matches!(&params.drag_state, DragState::Resizing { note_idx }      if *note_idx == i);

                let fill = if is_moving || is_resizing {
                    egui::Color32::from_rgba_premultiplied(
                        cr.saturating_add(40), cg.saturating_add(40), cb.saturating_add(40), 220,
                    )
                } else {
                    egui::Color32::from_rgb(cr, cg, cb)
                };
                painter.rect_filled(r, 2.0, fill);

                // Bright left-edge marker (note-on accent)
                painter.line_segment(
                    [egui::pos2(r.min.x, r.min.y), egui::pos2(r.min.x, r.max.y)],
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );

                // Resize handle — darker right strip
                let handle_rect = egui::Rect::from_min_max(
                    egui::pos2(r.max.x - RESIZE_HANDLE_PX, r.min.y),
                    r.max,
                );
                painter.rect_filled(handle_rect, 0.0, egui::Color32::from_rgba_premultiplied(0, 0, 0, 60));
            }

            // ── Playhead ──────────────────────────────────────────────────
            if params.enabled {
                let ph_x = grid_left + params.current_beat * px;
                painter.line_segment(
                    [egui::pos2(ph_x, grid_top - BEAT_HEADER_H), egui::pos2(ph_x, grid_bottom)],
                    egui::Stroke::new(1.5, egui::Color32::from_rgba_premultiplied(255, 220, 50, 200)),
                );
            }
        });

    // Row-level quick-fill ops
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Fill:");
        if ui.small_button("Default melody").clicked() {
            use crate::resources::programming_wheel_params::*;
            params.notes = marble_machine_default_notes_pub();
        }
        if ui.small_button("Kick quarter").clicked() {
            params.notes.retain(|n| n.channel != WHEEL_CH_CHUTE);
            let bpr = PROGRAMMING_WHEEL_BEATS_PER_REV as usize;
            for b in (0..bpr).step_by(1) {
                params.notes.push(WheelNote::new(WHEEL_CH_CHUTE, b as f32, 0.2));
            }
        }
        if ui.small_button("Snare 2+4").clicked() {
            params.notes.retain(|n| n.channel != WHEEL_CH_DROP);
            for bar in 0..16_usize {
                let b = bar as f32 * 4.0;
                params.notes.push(WheelNote::new(WHEEL_CH_DROP, b + 1.0, 0.2));
                params.notes.push(WheelNote::new(WHEEL_CH_DROP, b + 3.0, 0.2));
            }
        }
        if ui.small_button("Clear vibs").clicked() {
            params.notes.retain(|n| !matches!(channel_target(n.channel), ChannelTarget::VibBar { .. }));
        }
    });
}
