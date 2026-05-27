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

/// Full 16-bar (64-beat) programming wheel loop transcribed from the Wintergatan
/// Marble Machine MusicXML score. One revolution = 64 beats = 16 bars of 4/4 at
/// 120 BPM. Channels: ch = 2 + semitones_from_F3 (bar 0 = F3, 174.61 Hz).
///   B4=ch20  C5=ch21  D5=ch23  E5=ch25  F#5=ch27  G5=ch28
///   A5=ch30  B5=ch32  C6=ch33  D6=ch35  E6=ch37
fn marble_machine_default_notes() -> Vec<WheelNote> {
    let mut v: Vec<WheelNote> = Vec::new();

    // Kick (ch 0) on beats 0,2; snare (ch 1) on beats 1,3 — all 16 bars
    for bar in 0..16_usize {
        let b = bar as f32 * 4.0;
        v.push(WheelNote::new(WHEEL_CH_CHUTE, b + 0.0, 0.2));
        v.push(WheelNote::new(WHEEL_CH_DROP,  b + 1.0, 0.2));
        v.push(WheelNote::new(WHEEL_CH_CHUTE, b + 2.0, 0.2));
        v.push(WheelNote::new(WHEEL_CH_DROP,  b + 3.0, 0.2));
    }

    const B4: usize = 20; const C5: usize = 21; const D5: usize = 23;
    const E5: usize = 25; const FS5: usize = 27; const G5: usize = 28;
    const A5: usize = 30; const B5: usize = 32; const C6: usize = 33;
    const D6: usize = 35; const E6: usize = 37;

    {
        let (q, e) = (1.0_f32, 0.5_f32);
        let mut n = |beat: f32, ch: usize, len: f32| v.push(WheelNote::new(ch, beat, len));

        // ── Bars 1–2 (beats 0–7) ──────────────────────────────────────────
        n( 0.0, E6,  q);
        n( 1.0, E5,  e);  n( 1.0, B4,  q);  n( 1.5, B5,  e);
        n( 2.0, B5,  q);
        n( 3.0, E5,  e);  n( 3.0, B4,  q);  n( 3.5, A5,  e);

        n( 4.0, G5,  e);  n( 4.5, A5,  e);
        n( 5.0, E5,  e);  n( 5.0, B4,  q);  n( 5.5, B5,  e);
        n( 6.0, B5,  e);  n( 6.5, G5,  e);
        n( 7.0, A5,  e);  n( 7.0, B4,  q);  n( 7.5, D6,  e);
        n( 8.0, E5,  q);

        // ── Bars 3–4 (beats 8–15) ─────────────────────────────────────────
        n( 8.0, D6,  q);
        n( 9.0, D5,  e);  n( 9.0, B4,  q);  n( 9.5, B5,  e);
        n(10.0, B5,  q);
        n(11.0, D5,  e);  n(11.0, B4,  q);  n(11.5, A5,  e);

        n(12.0, G5,  e);  n(12.5, A5,  e);
        n(13.0, D5,  e);  n(13.0, B4,  q);  n(13.5, FS5, e);
        n(14.0, FS5, e);  n(14.5, G5,  e);
        n(15.0, A5,  e);  n(15.0, B4,  q);  n(15.5, D6,  e);
        n(16.0, D5,  q);

        // ── Bars 5–6 (beats 16–23) ────────────────────────────────────────
        n(16.0, D6,  q);
        n(17.0, FS5, e);  n(17.0, D5,  q);  n(17.5, B5,  e);
        n(18.0, B5,  q);
        n(19.0, FS5, e);  n(19.0, D5,  q);  n(19.5, D6,  e);

        n(20.0, C6,  e);  n(20.5, B5,  e);
        n(21.0, FS5, e);  n(21.0, D5,  q);  n(21.5, A5,  e);
        n(22.0, A5,  e);  n(22.5, G5,  e);
        n(23.0, A5,  e);  n(23.0, D5,  q);  n(23.5, E5,  e);
        n(24.0, FS5, q);

        // ── Bars 7–8 (beats 24–31) ────────────────────────────────────────
        n(24.0, E5,  e);  n(24.5, C5,  e);
        n(25.0, E5,  e);  n(25.5, B5,  e);
        n(26.0, B4,  e);  n(26.5, C5,  e);
        n(27.0, D5,  e);  n(27.5, D6,  e);

        n(28.0, C6,  e);  n(28.5, B5,  e);
        n(29.0, FS5, e);  n(29.0, D5,  q);  n(29.5, A5,  e);
        n(30.0, A5,  e);  n(30.5, G5,  e);
        n(31.0, A5,  e);  n(31.5, E6,  e);

        // ── Bars 9–10 (beats 32–39) ───────────────────────────────────────
        n(32.0, E6,  q);
        n(33.0, E5,  e);  n(33.0, B4,  q);  n(33.5, B5,  e);
        n(34.0, B5,  q);
        n(35.0, E5,  e);  n(35.0, B4,  q);  n(35.5, A5,  e);

        n(36.0, G5,  e);  n(36.5, A5,  e);
        n(37.0, E5,  e);  n(37.0, B4,  q);  n(37.5, B5,  e);
        n(38.0, B5,  e);  n(38.5, G5,  e);
        n(39.0, A5,  e);  n(39.0, B4,  q);  n(39.5, D6,  e);
        n(40.0, E5,  q);

        // ── Bars 11–12 (beats 40–47) ──────────────────────────────────────
        n(40.0, D6,  q);
        n(41.0, D5,  e);  n(41.0, B4,  q);  n(41.5, B5,  e);
        n(42.0, B5,  q);
        n(43.0, D5,  e);  n(43.0, B4,  q);  n(43.5, D6,  e);

        n(44.0, C6,  e);  n(44.5, B5,  e);
        n(45.0, D5,  e);  n(45.0, B4,  q);  n(45.5, A5,  e);
        n(46.0, A5,  e);  n(46.5, G5,  e);
        n(47.0, A5,  e);  n(47.0, B4,  q);  n(47.5, D6,  e);
        n(48.0, D5,  q);

        // ── Bars 13–14 (beats 48–55) ──────────────────────────────────────
        n(48.0, D6,  q);
        n(49.0, FS5, e);  n(49.0, D5,  q);  n(49.5, B5,  e);
        n(50.0, B5,  q);
        n(51.0, A5,  e);  n(51.0, D5,  q);  n(51.5, E6,  e);
        n(52.0, FS5, q);

        n(52.0, E6,  e);  n(52.5, B5,  e);
        n(53.0, D5,  e);  n(53.0, B4,  q);  n(53.5, A5,  e);
        n(54.0, A5,  e);  n(54.5, G5,  e);
        n(55.0, FS5, e);  n(55.0, B4,  q);  n(55.5, E5,  e);
        n(56.0, D5,  q);

        // ── Bars 15–16 (beats 56–63) ──────────────────────────────────────
        n(56.0, E5,  e);  n(56.5, B4,  e);
        n(57.0, C5,  e);  n(57.5, FS5, e);
        n(58.0, C5,  e);  n(58.5, E5,  e);
        n(59.0, G5,  e);  n(59.5, D5,  e);

        n(60.0, FS5, e);  n(60.5, A5,  e);
        n(61.0, B4,  e);  n(61.5, B5,  e);
        n(62.0, D5,  e);  n(62.5, G5,  e);
        n(63.0, A5,  e);  n(63.5, E6,  e);
    }

    v
}

/// Public alias so the UI fill button can reset to the default melody.
pub fn marble_machine_default_notes_pub() -> Vec<WheelNote> {
    marble_machine_default_notes()
}

struct ChannelDef {
    name:  &'static str,
    color: (u8, u8, u8),
}

const VIB: (u8, u8, u8) = (80, 200, 120);

/// Complete channel table, indexed by channel number.
/// Channels 0–1 are the non-pitched marble types.
/// Channels 2–38 are vibraphone bars 0–36 (F3 → F6, one semitone per step).
const CHANNEL_DEFS: &[ChannelDef] = &[
    ChannelDef { name: "Chute", color: (51, 115, 230) }, // 0
    ChannelDef { name: "Drop",  color: (242, 89,  38) }, // 1
    ChannelDef { name: "F3",   color: VIB }, // 2
    ChannelDef { name: "F#3",  color: VIB }, // 3
    ChannelDef { name: "G3",   color: VIB }, // 4
    ChannelDef { name: "G#3",  color: VIB }, // 5
    ChannelDef { name: "A3",   color: VIB }, // 6
    ChannelDef { name: "A#3",  color: VIB }, // 7
    ChannelDef { name: "B3",   color: VIB }, // 8
    ChannelDef { name: "C4",   color: VIB }, // 9
    ChannelDef { name: "C#4",  color: VIB }, // 10
    ChannelDef { name: "D4",   color: VIB }, // 11
    ChannelDef { name: "D#4",  color: VIB }, // 12
    ChannelDef { name: "E4",   color: VIB }, // 13
    ChannelDef { name: "F4",   color: VIB }, // 14
    ChannelDef { name: "F#4",  color: VIB }, // 15
    ChannelDef { name: "G4",   color: VIB }, // 16
    ChannelDef { name: "G#4",  color: VIB }, // 17
    ChannelDef { name: "A4",   color: VIB }, // 18
    ChannelDef { name: "A#4",  color: VIB }, // 19
    ChannelDef { name: "B4",   color: VIB }, // 20
    ChannelDef { name: "C5",   color: VIB }, // 21
    ChannelDef { name: "C#5",  color: VIB }, // 22
    ChannelDef { name: "D5",   color: VIB }, // 23
    ChannelDef { name: "D#5",  color: VIB }, // 24
    ChannelDef { name: "E5",   color: VIB }, // 25
    ChannelDef { name: "F5",   color: VIB }, // 26
    ChannelDef { name: "F#5",  color: VIB }, // 27
    ChannelDef { name: "G5",   color: VIB }, // 28
    ChannelDef { name: "G#5",  color: VIB }, // 29
    ChannelDef { name: "A5",   color: VIB }, // 30
    ChannelDef { name: "A#5",  color: VIB }, // 31
    ChannelDef { name: "B5",   color: VIB }, // 32
    ChannelDef { name: "C6",   color: VIB }, // 33
    ChannelDef { name: "C#6",  color: VIB }, // 34
    ChannelDef { name: "D6",   color: VIB }, // 35
    ChannelDef { name: "D#6",  color: VIB }, // 36
    ChannelDef { name: "E6",   color: VIB }, // 37
    ChannelDef { name: "F6",   color: VIB }, // 38
];

/// Returns the display name for a channel.
pub fn channel_name(ch: usize) -> String {
    CHANNEL_DEFS.get(ch).map_or("?".to_string(), |d| d.name.to_string())
}

/// Returns the (r, g, b) display color for a channel.
pub fn channel_color_rgb(ch: usize) -> (u8, u8, u8) {
    CHANNEL_DEFS.get(ch).map_or((128, 128, 128), |d| d.color)
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
