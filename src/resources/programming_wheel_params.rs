//! Programming wheel data: channel table, note sequence, and UI state.
//!
//! The `CHANNEL_DEFS` table is the single source of truth for every instrument channel.
//! Add new instruments there; everything else (`channel_name`, `channel_target`, etc.)
//! derives from it automatically.

use bevy::prelude::*;

use crate::resources::constants::*;

/// First chute channel index.  Channels `WHEEL_CH_CHUTE_FIRST + 0..N_CHUTES` map 1-to-1
/// to chute instances; each is a `GhostSnare` path.
pub const WHEEL_CH_CHUTE_FIRST: usize = 0;
/// Channel index of the vertical snare drop (direct fall).
pub const WHEEL_CH_DROP: usize = 6;
/// First vibraphone channel index. Used by spawner setup, hit detection, and the vibraphone component.
pub const WHEEL_CH_VIB_FIRST: usize = 13;
/// First hi-hat strike channel. Channels `WHEEL_CH_HIHAT_FIRST + 0..6` are the six hit zones.
pub const WHEEL_CH_HIHAT_FIRST: usize = 50;
/// Hi-hat pedal channel — controls the open/closed gate by beat position.
pub const WHEEL_CH_HIHAT_PEDAL: usize = 56;
/// First kick drum channel. Channels `WHEEL_CH_KICK_FIRST + 0..6` are the six hit zones.
pub const WHEEL_CH_KICK_FIRST: usize = 57;
/// First ride cymbal channel. Channels `WHEEL_CH_RIDE_FIRST + 0..6` are the six hit zones.
pub const WHEEL_CH_RIDE_FIRST: usize = 63;

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
/// 120 BPM. Channels: ch = WHEEL_CH_VIB_FIRST + semitones_from_F3
/// (bar 0 = F3, 174.61 Hz, ch 13).
///   B4=ch31  C5=ch32  D5=ch34  E5=ch36  F#5=ch38  G5=ch39
///   A5=ch41  B5=ch43  C6=ch44  D6=ch46  E6=ch48
pub fn marble_machine_default_notes() -> Vec<WheelNote> {
    let mut v: Vec<WheelNote> = Vec::new();

    // Chute 1 (ch 0) on beats 0,2; snare drop (ch 6) on beats 1,3 — all 16 bars
    for bar in 0..16_usize {
        let b = bar as f32 * 4.0;
        v.push(WheelNote::new(WHEEL_CH_CHUTE_FIRST, b + 0.0, 0.2));
        v.push(WheelNote::new(WHEEL_CH_DROP,        b + 1.0, 0.2));
        v.push(WheelNote::new(WHEEL_CH_CHUTE_FIRST, b + 2.0, 0.2));
        v.push(WheelNote::new(WHEEL_CH_DROP,        b + 3.0, 0.2));
        // Kick doubles the ghost snare on the strong beats (0 and 2)
        v.push(WheelNote::new(WHEEL_CH_KICK_FIRST,  b + 0.0, 0.2));
        v.push(WheelNote::new(WHEEL_CH_KICK_FIRST,  b + 2.0, 0.2));
        // Hi-hat drives every beat
        v.push(WheelNote::new(WHEEL_CH_HIHAT_FIRST, b + 0.0, 0.2));
        v.push(WheelNote::new(WHEEL_CH_HIHAT_FIRST, b + 1.0, 0.2));
        v.push(WheelNote::new(WHEEL_CH_HIHAT_FIRST, b + 2.0, 0.2));
        v.push(WheelNote::new(WHEEL_CH_HIHAT_FIRST, b + 3.0, 0.2));
        // Ride on the "and of 1" — 8th-note pulse underneath
        v.push(WheelNote::new(WHEEL_CH_RIDE_FIRST,  b + 0.5, 0.2));
    }

    // ch = WHEEL_CH_VIB_FIRST(13) + semitones_from_F3
    const B4:  usize = WHEEL_CH_VIB_FIRST + 18; const C5:  usize = WHEEL_CH_VIB_FIRST + 19;
    const D5:  usize = WHEEL_CH_VIB_FIRST + 21; const E5:  usize = WHEEL_CH_VIB_FIRST + 23;
    const FS5: usize = WHEEL_CH_VIB_FIRST + 25; const G5:  usize = WHEEL_CH_VIB_FIRST + 26;
    const A5:  usize = WHEEL_CH_VIB_FIRST + 28; const B5:  usize = WHEEL_CH_VIB_FIRST + 30;
    const C6:  usize = WHEEL_CH_VIB_FIRST + 31; const D6:  usize = WHEEL_CH_VIB_FIRST + 33;
    const E6:  usize = WHEEL_CH_VIB_FIRST + 35;

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


/// Which physical instrument a channel targets, and everything needed to spawn its marble.
/// This makes each channel self-describing — no arithmetic against `WHEEL_CH_VIB_FIRST`.
#[derive(Clone, Copy, PartialEq)]
pub enum ChannelTarget {
    /// Marble enters via the chute, then lands on the snare.
    GhostSnare,
    /// Marble drops directly onto the snare.
    /// `x_offset` is metres from the snare's world X position.
    Snare { x_offset: f32 },
    /// Marble drops onto vibraphone bar `bar_idx` (0 = F3, 36 = F6).
    VibBar { bar_idx: u32 },
    /// Marble drops onto the hi-hat cymbal. `x_offset` is metres from cymbal centre.
    HiHat { x_offset: f32 },
    /// Marble hits the hi-hat pedal, toggling the open/closed state.
    HiHatPedal,
    /// Marble drops onto the kick drum. `x_offset` is metres from drum centre.
    Kick { x_offset: f32 },
    /// Marble drops onto the ride cymbal. `x_offset` is metres from cymbal centre.
    Ride { x_offset: f32 },
}

struct ChannelDef {
    name:      &'static str,
    color:     (u8, u8, u8),
    target:    ChannelTarget,
    /// XZ jitter radius (metres) applied at marble spawn time.  0 = none.
    jitter_xz: f32,
}

const VIB:   (u8, u8, u8) = (80, 200, 120);
const SNARE: (u8, u8, u8) = (242, 89, 38);
const HIHAT: (u8, u8, u8) = (200, 165, 40);
const KICK:  (u8, u8, u8) = (180, 110, 55);
const RIDE:  (u8, u8, u8) = (210, 175, 60);
/// Lateral XZ jitter for snare drops (realistic marble-release noise).
const SNARE_JITTER: f32 = crate::resources::constants::MARBLE_SPAWN_JITTER;

/// Complete instrument channel table, indexed by channel number.
///
/// Each entry defines one instrument (or delivery path):
/// - Channels 0–5:   Ghost snare chute channels.
/// - Channels 6–12:  Direct snare drops at increasing X offsets (centre ± 2/4/6 cm).
/// - Channels 13–49: Vibraphone bars 0–36 (F3 → F6).
/// - Channels 50–55: Hi-hat strike zones (centre ± 2/4/6 cm).
/// - Channel 56:     Hi-hat pedal (gate, no marble).
/// - Channels 57–62: Kick drum (centre ± 2/4/6 cm).
/// - Channels 63–68: Ride cymbal (centre ± 2/4/6 cm).
///
/// To add a new instrument: append a `ChannelDef` here, spawn an `Instrument`
/// entity with the matching `channel`, and update `sync_instrument_spawners`.
const CHANNEL_DEFS: &[ChannelDef] = &[
    // ch 0–5 — chute channels (one per chute instance, GhostSnare path)
    ChannelDef { name: "Ghost Snare 1", color: (51, 115, 230), target: ChannelTarget::GhostSnare, jitter_xz: 0.0 }, // 0
    ChannelDef { name: "Ghost Snare 2", color: (70, 130, 210), target: ChannelTarget::GhostSnare, jitter_xz: 0.0 }, // 1
    ChannelDef { name: "Ghost Snare 3", color: (90, 150, 230), target: ChannelTarget::GhostSnare, jitter_xz: 0.0 }, // 2
    ChannelDef { name: "Ghost Snare 4", color: (40, 100, 215), target: ChannelTarget::GhostSnare, jitter_xz: 0.0 }, // 3
    ChannelDef { name: "Ghost Snare 5", color: (60,  85, 200), target: ChannelTarget::GhostSnare, jitter_xz: 0.0 }, // 4
    ChannelDef { name: "Ghost Snare 6", color: (80, 160, 225), target: ChannelTarget::GhostSnare, jitter_xz: 0.0 }, // 5
    // ch 6–12 — direct snare drops
    ChannelDef { name: "Snare",   color: SNARE,          target: ChannelTarget::Snare { x_offset:  0.00 }, jitter_xz: SNARE_JITTER }, // 6  centre
    ChannelDef { name: "Snare+2", color: SNARE,          target: ChannelTarget::Snare { x_offset:  0.02 }, jitter_xz: SNARE_JITTER }, // 7
    ChannelDef { name: "Snare-2", color: SNARE,          target: ChannelTarget::Snare { x_offset: -0.02 }, jitter_xz: SNARE_JITTER }, // 8
    ChannelDef { name: "Snare+4", color: SNARE,          target: ChannelTarget::Snare { x_offset:  0.04 }, jitter_xz: SNARE_JITTER }, // 9
    ChannelDef { name: "Snare-4", color: SNARE,          target: ChannelTarget::Snare { x_offset: -0.04 }, jitter_xz: SNARE_JITTER }, // 10
    ChannelDef { name: "Snare+6", color: SNARE,          target: ChannelTarget::Snare { x_offset:  0.06 }, jitter_xz: SNARE_JITTER }, // 11
    ChannelDef { name: "Snare-6", color: SNARE,          target: ChannelTarget::Snare { x_offset: -0.06 }, jitter_xz: SNARE_JITTER }, // 12
    // ch 13–49 — vibraphone bars 0–36 (F3 → F6)
    ChannelDef { name: "F3",  color: VIB, target: ChannelTarget::VibBar { bar_idx:  0 }, jitter_xz: 0.0 }, // 13
    ChannelDef { name: "F#3", color: VIB, target: ChannelTarget::VibBar { bar_idx:  1 }, jitter_xz: 0.0 }, // 14
    ChannelDef { name: "G3",  color: VIB, target: ChannelTarget::VibBar { bar_idx:  2 }, jitter_xz: 0.0 }, // 15
    ChannelDef { name: "G#3", color: VIB, target: ChannelTarget::VibBar { bar_idx:  3 }, jitter_xz: 0.0 }, // 16
    ChannelDef { name: "A3",  color: VIB, target: ChannelTarget::VibBar { bar_idx:  4 }, jitter_xz: 0.0 }, // 17
    ChannelDef { name: "A#3", color: VIB, target: ChannelTarget::VibBar { bar_idx:  5 }, jitter_xz: 0.0 }, // 18
    ChannelDef { name: "B3",  color: VIB, target: ChannelTarget::VibBar { bar_idx:  6 }, jitter_xz: 0.0 }, // 19
    ChannelDef { name: "C4",  color: VIB, target: ChannelTarget::VibBar { bar_idx:  7 }, jitter_xz: 0.0 }, // 20
    ChannelDef { name: "C#4", color: VIB, target: ChannelTarget::VibBar { bar_idx:  8 }, jitter_xz: 0.0 }, // 21
    ChannelDef { name: "D4",  color: VIB, target: ChannelTarget::VibBar { bar_idx:  9 }, jitter_xz: 0.0 }, // 22
    ChannelDef { name: "D#4", color: VIB, target: ChannelTarget::VibBar { bar_idx: 10 }, jitter_xz: 0.0 }, // 23
    ChannelDef { name: "E4",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 11 }, jitter_xz: 0.0 }, // 24
    ChannelDef { name: "F4",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 12 }, jitter_xz: 0.0 }, // 25
    ChannelDef { name: "F#4", color: VIB, target: ChannelTarget::VibBar { bar_idx: 13 }, jitter_xz: 0.0 }, // 26
    ChannelDef { name: "G4",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 14 }, jitter_xz: 0.0 }, // 27
    ChannelDef { name: "G#4", color: VIB, target: ChannelTarget::VibBar { bar_idx: 15 }, jitter_xz: 0.0 }, // 28
    ChannelDef { name: "A4",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 16 }, jitter_xz: 0.0 }, // 29
    ChannelDef { name: "A#4", color: VIB, target: ChannelTarget::VibBar { bar_idx: 17 }, jitter_xz: 0.0 }, // 30
    ChannelDef { name: "B4",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 18 }, jitter_xz: 0.0 }, // 31
    ChannelDef { name: "C5",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 19 }, jitter_xz: 0.0 }, // 32
    ChannelDef { name: "C#5", color: VIB, target: ChannelTarget::VibBar { bar_idx: 20 }, jitter_xz: 0.0 }, // 33
    ChannelDef { name: "D5",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 21 }, jitter_xz: 0.0 }, // 34
    ChannelDef { name: "D#5", color: VIB, target: ChannelTarget::VibBar { bar_idx: 22 }, jitter_xz: 0.0 }, // 35
    ChannelDef { name: "E5",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 23 }, jitter_xz: 0.0 }, // 36
    ChannelDef { name: "F5",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 24 }, jitter_xz: 0.0 }, // 37
    ChannelDef { name: "F#5", color: VIB, target: ChannelTarget::VibBar { bar_idx: 25 }, jitter_xz: 0.0 }, // 38
    ChannelDef { name: "G5",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 26 }, jitter_xz: 0.0 }, // 39
    ChannelDef { name: "G#5", color: VIB, target: ChannelTarget::VibBar { bar_idx: 27 }, jitter_xz: 0.0 }, // 40
    ChannelDef { name: "A5",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 28 }, jitter_xz: 0.0 }, // 41
    ChannelDef { name: "A#5", color: VIB, target: ChannelTarget::VibBar { bar_idx: 29 }, jitter_xz: 0.0 }, // 42
    ChannelDef { name: "B5",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 30 }, jitter_xz: 0.0 }, // 43
    ChannelDef { name: "C6",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 31 }, jitter_xz: 0.0 }, // 44
    ChannelDef { name: "C#6", color: VIB, target: ChannelTarget::VibBar { bar_idx: 32 }, jitter_xz: 0.0 }, // 45
    ChannelDef { name: "D6",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 33 }, jitter_xz: 0.0 }, // 46
    ChannelDef { name: "D#6", color: VIB, target: ChannelTarget::VibBar { bar_idx: 34 }, jitter_xz: 0.0 }, // 47
    ChannelDef { name: "E6",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 35 }, jitter_xz: 0.0 }, // 48
    ChannelDef { name: "F6",  color: VIB, target: ChannelTarget::VibBar { bar_idx: 36 }, jitter_xz: 0.0 }, // 49
    // ch 50–55 — hi-hat strike (6 zones spread over ±6 cm)
    ChannelDef { name: "Hi-Hat",   color: HIHAT, target: ChannelTarget::HiHat { x_offset:  0.00 }, jitter_xz: 0.001 }, // 50
    ChannelDef { name: "Hi-Hat+2", color: HIHAT, target: ChannelTarget::HiHat { x_offset:  0.02 }, jitter_xz: 0.001 }, // 51
    ChannelDef { name: "Hi-Hat-2", color: HIHAT, target: ChannelTarget::HiHat { x_offset: -0.02 }, jitter_xz: 0.001 }, // 52
    ChannelDef { name: "Hi-Hat+4", color: HIHAT, target: ChannelTarget::HiHat { x_offset:  0.04 }, jitter_xz: 0.001 }, // 53
    ChannelDef { name: "Hi-Hat-4", color: HIHAT, target: ChannelTarget::HiHat { x_offset: -0.04 }, jitter_xz: 0.001 }, // 54
    ChannelDef { name: "Hi-Hat+6", color: HIHAT, target: ChannelTarget::HiHat { x_offset:  0.06 }, jitter_xz: 0.001 }, // 55
    // ch 56 — hi-hat pedal (gate, no marble)
    ChannelDef { name: "HH Pedal", color: (150, 120, 30), target: ChannelTarget::HiHatPedal, jitter_xz: 0.0 }, // 56
    // ch 57–62 — kick drum (6 zones spread over ±6 cm)
    ChannelDef { name: "Kick",   color: KICK, target: ChannelTarget::Kick { x_offset:  0.00 }, jitter_xz: 0.001 }, // 57
    ChannelDef { name: "Kick+2", color: KICK, target: ChannelTarget::Kick { x_offset:  0.02 }, jitter_xz: 0.001 }, // 58
    ChannelDef { name: "Kick-2", color: KICK, target: ChannelTarget::Kick { x_offset: -0.02 }, jitter_xz: 0.001 }, // 59
    ChannelDef { name: "Kick+4", color: KICK, target: ChannelTarget::Kick { x_offset:  0.04 }, jitter_xz: 0.001 }, // 60
    ChannelDef { name: "Kick-4", color: KICK, target: ChannelTarget::Kick { x_offset: -0.04 }, jitter_xz: 0.001 }, // 61
    ChannelDef { name: "Kick+6", color: KICK, target: ChannelTarget::Kick { x_offset:  0.06 }, jitter_xz: 0.001 }, // 62
    // ch 63–68 — ride cymbal (6 zones spread over ±6 cm)
    ChannelDef { name: "Ride",   color: RIDE, target: ChannelTarget::Ride { x_offset:  0.00 }, jitter_xz: 0.001 }, // 63
    ChannelDef { name: "Ride+2", color: RIDE, target: ChannelTarget::Ride { x_offset:  0.02 }, jitter_xz: 0.001 }, // 64
    ChannelDef { name: "Ride-2", color: RIDE, target: ChannelTarget::Ride { x_offset: -0.02 }, jitter_xz: 0.001 }, // 65
    ChannelDef { name: "Ride+4", color: RIDE, target: ChannelTarget::Ride { x_offset:  0.04 }, jitter_xz: 0.001 }, // 66
    ChannelDef { name: "Ride-4", color: RIDE, target: ChannelTarget::Ride { x_offset: -0.04 }, jitter_xz: 0.001 }, // 67
    ChannelDef { name: "Ride+6", color: RIDE, target: ChannelTarget::Ride { x_offset:  0.06 }, jitter_xz: 0.001 }, // 68
];

/// Returns the display name for a channel.
pub fn channel_name(ch: usize) -> String {
    CHANNEL_DEFS.get(ch).map_or("?".to_string(), |d| d.name.to_string())
}

/// Returns the (r, g, b) display color for a channel.
pub fn channel_color_rgb(ch: usize) -> (u8, u8, u8) {
    CHANNEL_DEFS.get(ch).map_or((128, 128, 128), |d| d.color)
}

/// Returns the target instrument and spawn parameters for a channel.
pub fn channel_target(ch: usize) -> ChannelTarget {
    CHANNEL_DEFS.get(ch).map_or(ChannelTarget::Snare { x_offset: 0.0 }, |d| d.target)
}

/// Returns the XZ jitter radius (metres) for a channel's marble spawn.
/// Returns 0 for unknown channels.
pub fn channel_jitter_xz(ch: usize) -> f32 {
    CHANNEL_DEFS.get(ch).map_or(0.0, |d| d.jitter_xz)
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
