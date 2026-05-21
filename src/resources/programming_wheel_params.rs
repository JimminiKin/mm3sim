use bevy::prelude::*;

use crate::resources::constants::*;

/// Channel 0  = chute drop marble
/// Channel 1  = vertical snare drop marble
/// Channel 2..=38 = vibraphone bars 0..=36
pub const WHEEL_CH_CHUTE: usize = 0;
pub const WHEEL_CH_DROP: usize = 1;
pub const WHEEL_CH_VIB_FIRST: usize = 2;

/// A single MIDI-style note on the programming wheel.
/// `beat` is the start position in beats [0, BEATS_PER_REV).
/// `length` is the duration in beats (stored for future use; marble fires on beat start).
#[derive(Clone, Debug, PartialEq)]
pub struct WheelNote {
    pub channel: usize,
    pub beat: f32,
    pub length: f32,
}

impl WheelNote {
    pub fn new(channel: usize, beat: f32, length: f32) -> Self {
        Self { channel, beat, length }
    }
}

/// State of the piano-roll drag interaction.
#[derive(Default, Clone, Debug)]
pub enum DragState {
    #[default]
    None,
    /// Left-drag on empty space: creating a new note.
    Creating { channel: usize, start_beat: f32, end_beat: f32 },
    /// Left-drag on a note body: moving it horizontally.
    Moving { note_idx: usize, beat_offset: f32 },
    /// Left-drag on a note's right edge: resizing its length.
    Resizing { note_idx: usize },
}

#[derive(Resource)]
pub struct ProgrammingWheelParams {
    pub enabled: bool,
    pub rpm: f32,
    /// Current wheel rotation in radians [0, 2π).
    pub angle: f32,
    /// Current playhead position in beats [0, BEATS_PER_REV).
    pub current_beat: f32,
    /// All programmed notes.
    pub notes: Vec<WheelNote>,
    /// Piano-roll drag state.
    pub drag_state: DragState,
    pub show_pegs: bool,
    /// Channel indices queued for marble spawning this frame.
    pub pending_spawns: Vec<usize>,
    /// Piano-roll UI zoom: pixels per beat.
    pub px_per_beat: f32,
    /// Quantise grid in beats (0 = free, 0.25 = 16th, etc.).
    pub snap_beats: f32,
}

impl Default for ProgrammingWheelParams {
    fn default() -> Self {
        Self {
            enabled: false,
            rpm: PROGRAMMING_WHEEL_RPM_DEFAULT,
            angle: 0.0,
            current_beat: 0.0,
            notes: marble_machine_default_notes(),
            drag_state: DragState::None,
            show_pegs: true,
            pending_spawns: Vec::new(),
            px_per_beat: 15.0,
            snap_beats: 0.25,
        }
    }
}

impl ProgrammingWheelParams {
    pub fn reset_position(&mut self) {
        self.angle = 0.0;
        self.current_beat = 0.0;
    }

    pub fn clear_notes(&mut self) {
        self.notes.clear();
    }
}

/// Wintergatan "Marble Machine" opening melody, converted to beat positions.
///
/// 1 revolution = 64 beats = 16 bars of 4/4 at 120 BPM (1.875 RPM).
/// Old step positions divided by 12 give beat positions.
///
/// Vibraphone channels: bar_index + 2 (WHEEL_CH_VIB_FIRST).
///   E4=ch13  F#4=ch15  G4=ch16  A4=ch18  B4=ch20
///   C5=ch21  D5=ch23   F#5=ch27 B5=ch32
fn marble_machine_default_notes() -> Vec<WheelNote> {
    let mut v: Vec<WheelNote> = Vec::new();

    let mel  = |beat: f32, ch: usize| WheelNote::new(ch, beat, 0.45);
    let perc = |beat: f32, ch: usize| WheelNote::new(ch, beat, 0.2);
    let tri  = |beat: f32, ch: usize| WheelNote::new(ch, beat, 0.28); // triplet-8th length

    // ── Percussion: kick ch0 on beats 0,2; snare ch1 on beats 1,3 — 4 bars ──
    for bar in 0..4_usize {
        let b = bar as f32 * 4.0;
        v.push(perc(b + 0.0, WHEEL_CH_CHUTE));
        v.push(perc(b + 1.0, WHEEL_CH_DROP));
        v.push(perc(b + 2.0, WHEEL_CH_CHUTE));
        v.push(perc(b + 3.0, WHEEL_CH_DROP));
    }

    // ── Bar 1 ─ "e e b e  a g a  e" ─────────────────────────────────────────
    v.push(mel(0.00, 13)); // E4 — 16th pair opener
    v.push(mel(0.25, 13)); // E4
    v.push(mel(1.00, 20)); // B4 — beat 2
    v.push(mel(1.50, 13)); // E4
    v.push(mel(2.00, 18)); // A4
    v.push(mel(2.25, 16)); // G4 — 16th pair
    v.push(mel(2.75, 18)); // A4
    v.push(mel(3.50, 13)); // E4 — dotted-8th gap

    // ── Bar 2 ─ "b g  a d d  b d a" ─────────────────────────────────────────
    v.push(mel(4.00, 20)); // B4
    v.push(mel(4.50, 16)); // G4
    v.push(mel(5.00, 18)); // A4
    v.push(mel(5.50, 23)); // D5
    v.push(mel(5.75, 23)); // D5 — 16th doublet
    v.push(mel(6.50, 20)); // B4
    v.push(mel(7.00, 23)); // D5
    v.push(mel(7.50, 18)); // A4

    // ── Bar 3 ─ "g a d F#  g a d F#" — triplet 8ths then 8th notes ──────────
    let t = 1.0_f32 / 3.0; // triplet 8th = 1/3 beat
    v.push(tri(8.00,         16)); // G4
    v.push(tri(8.00 + t,     18)); // A4
    v.push(tri(8.00 + 2.0*t, 23)); // D5
    v.push(mel(9.00, 27));         // F#5
    v.push(mel( 9.50, 16));        // G4
    v.push(mel(10.00, 18));        // A4
    v.push(mel(10.50, 23));        // D5
    v.push(mel(11.00, 27));        // F#5

    // ── Bar 4 ─ "b5 F#5 d c  b F#4 a g" — climax + descent ──────────────────
    // 1-beat rest before B5 climax (steps 132-143 → beats 11-12)
    v.push(mel(12.00, 32)); // B5 — climax
    v.push(mel(12.50, 27)); // F#5
    v.push(mel(13.00, 23)); // D5
    v.push(mel(13.50, 21)); // C5
    v.push(mel(14.00, 20)); // B4
    v.push(mel(14.50, 15)); // F#4
    v.push(mel(15.00, 18)); // A4
    v.push(mel(15.50, 16)); // G4

    v
}

/// Public alias so the UI fill button can reset to the default melody.
pub fn marble_machine_default_notes_pub() -> Vec<WheelNote> {
    marble_machine_default_notes()
}

pub fn channel_name(ch: usize) -> &'static str {
    match ch {
        WHEEL_CH_CHUTE => "Chute",
        WHEEL_CH_DROP => "Drop",
        2  => "Vib 00", 3  => "Vib 01", 4  => "Vib 02", 5  => "Vib 03",
        6  => "Vib 04", 7  => "Vib 05", 8  => "Vib 06", 9  => "Vib 07",
        10 => "Vib 08", 11 => "Vib 09", 12 => "Vib 10", 13 => "Vib 11",
        14 => "Vib 12", 15 => "Vib 13", 16 => "Vib 14", 17 => "Vib 15",
        18 => "Vib 16", 19 => "Vib 17", 20 => "Vib 18", 21 => "Vib 19",
        22 => "Vib 20", 23 => "Vib 21", 24 => "Vib 22", 25 => "Vib 23",
        26 => "Vib 24", 27 => "Vib 25", 28 => "Vib 26", 29 => "Vib 27",
        30 => "Vib 28", 31 => "Vib 29", 32 => "Vib 30", 33 => "Vib 31",
        34 => "Vib 32", 35 => "Vib 33", 36 => "Vib 34", 37 => "Vib 35",
        38 => "Vib 36",
        _ => "?",
    }
}

pub fn channel_color_rgb(ch: usize) -> (u8, u8, u8) {
    match ch {
        WHEEL_CH_CHUTE => (51, 115, 230),
        WHEEL_CH_DROP  => (242, 89, 38),
        _              => (80, 200, 120),
    }
}

pub fn snap_beat(beat: f32, snap: f32) -> f32 {
    if snap <= 0.0 { beat } else { (beat / snap).round() * snap }
}

pub fn snap_label(snap: f32) -> &'static str {
    if snap <= 0.0            { "Free" }
    else if (snap - 0.25).abs() < 0.01 { "1/16" }
    else if (snap - 1.0/3.0).abs() < 0.01 { "1/8T" }
    else if (snap - 0.5).abs() < 0.01  { "1/8"  }
    else if (snap - 1.0).abs() < 0.01  { "1/4"  }
    else                               { "Custom" }
}
