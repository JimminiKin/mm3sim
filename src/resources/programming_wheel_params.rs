use bevy::prelude::*;

use crate::resources::constants::*;

/// Channel 0  = chute drop marble
/// Channel 1  = vertical snare drop marble
/// Channel 2..=38 = vibraphone bars 0..=36
pub const WHEEL_CH_CHUTE: usize = 0;
pub const WHEEL_CH_DROP: usize = 1;
pub const WHEEL_CH_VIB_FIRST: usize = 2;

#[derive(Resource)]
pub struct ProgrammingWheelParams {
    pub enabled: bool,
    pub rpm: f32,
    /// Current wheel rotation in radians [0, 2π).
    /// Initialised to just before step-0 so that step 0 fires on the first
    /// enabled frame rather than only after a full revolution.
    pub angle: f32,
    /// Step index currently aligned with the reader bar (updated each frame).
    pub current_step: usize,
    /// Active pattern: pattern[step][channel]
    pub pattern: Vec<Vec<bool>>,
    /// Drag-paint state for the pattern editor
    pub drag_paint_val: Option<bool>,
    pub show_pegs: bool,
    /// Pending (step, channel) triggers written by rotate_programming_wheel_system,
    /// drained by programming_wheel_spawn_system each frame.
    pub pending_spawns: Vec<(usize, usize)>,
}

impl Default for ProgrammingWheelParams {
    fn default() -> Self {
        // Start just before step 0 so it fires immediately when play begins
        let start_angle =
            std::f32::consts::TAU * (1.0 - 0.005 / PROGRAMMING_WHEEL_N_STEPS as f32);
        Self {
            enabled: false,
            rpm: PROGRAMMING_WHEEL_RPM_DEFAULT,
            angle: start_angle,
            current_step: 0,
            pattern: marble_machine_default_pattern(),
            drag_paint_val: None,
            show_pegs: true,
            pending_spawns: Vec::new(),
        }
    }
}

/// Wintergatan "Marble Machine" opening vibraphone melody.
///
/// Key: E minor, 120 BPM, 4/4 time.
///
/// Source: the opening 32 8th-note phrase from the song.  Each letter is one
/// 8th note; uppercase = sharp (F = F#).  Sequence:
///   e e b e  a g a e  | b g a d  d b d a  |
///   g a d F  g a d F  | b F d c  b F a g  |
/// (climbs to B5 on bar-3 beat-1 then descends back to G4)
///
/// Vibraphone tuning: bar 0 = F3 (174.61 Hz), one semitone per bar.
/// Channel = bar_index + 2 (WHEEL_CH_VIB_FIRST).
///
///   E4 = bar 11 → ch 13    F#4 = bar 13 → ch 15   G4 = bar 14 → ch 16
///   A4 = bar 16 → ch 18    B4  = bar 18 → ch 20   C5 = bar 19 → ch 21
///   D5 = bar 21 → ch 23    F#5 = bar 25 → ch 27   B5 = bar 30 → ch 32
///
/// Percussion: kick (chute) on beats 1 & 3, snare (drop) on beats 2 & 4.
/// 32 notes × 6 steps = 192 steps, filling the loop exactly.
fn marble_machine_default_pattern() -> Vec<Vec<bool>> {
    let mut p = vec![vec![false; PROGRAMMING_WHEEL_N_CHANNELS]; PROGRAMMING_WHEEL_N_STEPS];

    // Kick on beats 1 & 3, snare on beats 2 & 4 — 4 bars × 4 beats
    let bar_steps = 4 * PROGRAMMING_WHEEL_STEPS_PER_BEAT; // 48 steps per bar
    for bar in 0..4_usize {
        let bs = bar * bar_steps;
        p[bs][WHEEL_CH_CHUTE] = true;                                    // beat 1 kick
        p[bs + PROGRAMMING_WHEEL_STEPS_PER_BEAT][WHEEL_CH_DROP] = true; // beat 2 snare
        p[bs + 2 * PROGRAMMING_WHEEL_STEPS_PER_BEAT][WHEEL_CH_CHUTE] = true; // beat 3 kick
        p[bs + 3 * PROGRAMMING_WHEEL_STEPS_PER_BEAT][WHEEL_CH_DROP] = true;  // beat 4 snare
    }

    // Vibraphone melody: 32 8th notes (every 6 steps).
    // Each entry is the vib channel for that 8th-note position.
    // 32 × 6 = 192 steps — fills the loop exactly.
    // Melody placed at specific step positions to capture the original song's
    // irregular phrasing.  Spacing rules per bar:
    //   Bar 1: 16th-note opener (3 steps) + 8th notes with a dotted-8th gap
    //   Bar 2: 8th notes + a 16th-note D5 double-tap ("d d" in the transcription)
    //   Bar 3: triplet-8th ascent (every 4 steps) then 8th notes
    //   Bar 4: 8th-note descent from the B5 peak — 1 beat rest before the climb
    #[rustfmt::skip]
    let notes: &[(usize, usize)] = &[
        // ── Bar 1 ─ "e e b e  a g a  e" ───────────────────────────────────────
        ( 0, 13), ( 3, 13),               // E4 E4  — 16th pair opener
        (12, 20), (18, 13),               // B4 E4  — beat 2, and-2
        (24, 18), (27, 16), (33, 18),     // A4 G4 A4 — 16th pair then 8th
        (42, 13),                         // E4     — and-4 (dotted-8th gap)

        // ── Bar 2 ─ "b g  a d d  b d a" ───────────────────────────────────────
        (48, 20), (54, 16),               // B4 G4
        (60, 18), (66, 23), (69, 23),     // A4 D5 D5 — 8th + 16th pair ("d d" doublet)
        (78, 20), (84, 23), (90, 18),     // B4 D5 A4 — dotted-8th gap then 8ths

        // ── Bar 3 ─ "g a d F#  g a d F#" — triplet-8th ascent then 8th notes ─
        ( 96, 16), (100, 18), (104, 23), (108, 27), // G4 A4 D5 F#5 — triplet 8ths
        (114, 16), (120, 18), (126, 23), (132, 27), // G4 A4 D5 F#5 — 8th notes

        // ── Bar 4 ─ "b5 F#5 d c  b F#4 a g" — peak + smooth descent ──────────
        // 1-beat rest (steps 133-143) before the B5 climax on beat 1
        (144, 32), (150, 27),             // B5 F#5 — climax (coincides with kick)
        (156, 23), (162, 21),             // D5 C5
        (168, 20), (174, 15),             // B4 F#4
        (180, 18), (186, 16),             // A4 G4
    ];

    for &(step, ch) in notes {
        if step < PROGRAMMING_WHEEL_N_STEPS && ch < PROGRAMMING_WHEEL_N_CHANNELS {
            p[step][ch] = true;
        }
    }

    p
}

impl ProgrammingWheelParams {
    pub fn reset_position(&mut self) {
        self.angle =
            std::f32::consts::TAU * (1.0 - 0.005 / PROGRAMMING_WHEEL_N_STEPS as f32);
        self.current_step = 0;
    }

    pub fn clear_pattern(&mut self) {
        for row in &mut self.pattern {
            for cell in row.iter_mut() {
                *cell = false;
            }
        }
    }
}

pub fn channel_name(ch: usize) -> &'static str {
    match ch {
        WHEEL_CH_CHUTE => "Chute",
        WHEEL_CH_DROP => "Drop",
        2 => "Vib 00",
        3 => "Vib 01",
        4 => "Vib 02",
        5 => "Vib 03",
        6 => "Vib 04",
        7 => "Vib 05",
        8 => "Vib 06",
        9 => "Vib 07",
        10 => "Vib 08",
        11 => "Vib 09",
        12 => "Vib 10",
        13 => "Vib 11",
        14 => "Vib 12",
        15 => "Vib 13",
        16 => "Vib 14",
        17 => "Vib 15",
        18 => "Vib 16",
        19 => "Vib 17",
        20 => "Vib 18",
        21 => "Vib 19",
        22 => "Vib 20",
        23 => "Vib 21",
        24 => "Vib 22",
        25 => "Vib 23",
        26 => "Vib 24",
        27 => "Vib 25",
        28 => "Vib 26",
        29 => "Vib 27",
        30 => "Vib 28",
        31 => "Vib 29",
        32 => "Vib 30",
        33 => "Vib 31",
        34 => "Vib 32",
        35 => "Vib 33",
        36 => "Vib 34",
        37 => "Vib 35",
        38 => "Vib 36",
        _ => "?",
    }
}

/// Colour used in the 2-D pattern grid for each channel group.
pub fn channel_color_rgb(ch: usize) -> (u8, u8, u8) {
    match ch {
        WHEEL_CH_CHUTE => (51, 115, 230),  // blue – chute marble
        WHEEL_CH_DROP => (242, 89, 38),    // orange – drop marble
        _ => (80, 200, 120),               // green – vibraphone
    }
}
