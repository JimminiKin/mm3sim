use bevy::prelude::*;

use crate::resources::constants::*;

/// Channel 0  = chute drop marble
/// Channel 1  = vertical snare drop marble
/// Channel 2..=38 = vibraphone bars 0..=36
pub const BARREL_CH_CHUTE: usize = 0;
pub const BARREL_CH_DROP: usize = 1;
pub const BARREL_CH_VIB_FIRST: usize = 2;

#[derive(Resource)]
pub struct BarrelParams {
    pub enabled: bool,
    pub rpm: f32,
    /// Current barrel rotation in radians [0, 2π).
    /// Initialised to just before step-0 so that step 0 fires on the first
    /// enabled frame rather than only after a full revolution.
    pub angle: f32,
    /// Step index currently aligned with the reader bar (updated each frame).
    pub current_step: usize,
    /// Active pattern: pattern[step][channel]
    pub pattern: Vec<Vec<bool>>,
    /// Drag-paint state for the pattern editor
    pub drag_paint_val: Option<bool>,
    pub editor_open: bool,
    pub show_pegs: bool,
    /// Pending (step, channel) triggers written by rotate_barrel_system,
    /// drained by barrel_spawn_system each frame.
    pub pending_spawns: Vec<(usize, usize)>,
}

impl Default for BarrelParams {
    fn default() -> Self {
        // Start just before step 0 so it fires immediately when play begins
        let start_angle =
            std::f32::consts::TAU * (1.0 - 0.005 / BARREL_N_STEPS as f32);
        Self {
            enabled: false,
            rpm: BARREL_RPM_DEFAULT,
            angle: start_angle,
            current_step: 0,
            pattern: marble_machine_default_pattern(),
            drag_paint_val: None,
            editor_open: false,
            show_pegs: true,
            pending_spawns: Vec::new(),
        }
    }
}

/// Default pattern approximating the Marble Machine (Wintergatan) main loop.
///
/// Vibraphone tuning: bar 0 = F3 (174.61 Hz), chromatic semitones up.
/// Channel = bar_index + BARREL_CH_VIB_FIRST (2).
///
/// Notes used (bar → channel):
///   A3=4→6  D4=9→11  E4=11→13  F4=12→14  G4=14→16
///   A4=16→18  C5=19→21  D5=21→23  F5=24→26  A5=28→30
///
/// Structure (192 steps = 16 beats = 4 bars of 4/4 at 120 BPM):
///   Kick (ch 0): every beat (quarter notes)
///   Snare (ch 1): beats 2 & 4 of each bar (backbeat)
///   Vibraphone: D-minor ascending arpeggio then descending scale, 8-beat
///               phrase repeated twice.
fn marble_machine_default_pattern() -> Vec<Vec<bool>> {
    let mut p = vec![vec![false; BARREL_N_CHANNELS]; BARREL_N_STEPS];

    // Kick: chute on every quarter-note beat
    for beat in 0..16_usize {
        p[beat * BARREL_STEPS_PER_BEAT][BARREL_CH_CHUTE] = true;
    }

    // Snare: drop on beats 2 & 4 of every 4-beat bar (4 bars in the loop)
    for bar in 0..4_usize {
        let bar_start = bar * (4 * BARREL_STEPS_PER_BEAT);
        p[bar_start + BARREL_STEPS_PER_BEAT][BARREL_CH_DROP] = true;     // beat 2
        p[bar_start + 3 * BARREL_STEPS_PER_BEAT][BARREL_CH_DROP] = true; // beat 4
    }

    // Vibraphone: 8th-note melody (every 6 steps), 8-beat phrase × 2 reps.
    // Phrase = ascending Dm7 arpeggio (D4-F4-A4-C5-D5-F5-A5-F5) then
    //          descending scale (D5-C5-A4-G4-F4-E4-D4-A3).
    // (step_offset_within_phrase, vib_channel)
    const HALF: usize = 6; // 8th note = 6 steps
    let phrase: &[(usize, usize)] = &[
        (0 * HALF, 11), // D4
        (1 * HALF, 14), // F4
        (2 * HALF, 18), // A4
        (3 * HALF, 21), // C5
        (4 * HALF, 23), // D5
        (5 * HALF, 26), // F5
        (6 * HALF, 30), // A5
        (7 * HALF, 26), // F5  ← pivot
        (8 * HALF, 23), // D5
        (9 * HALF, 21), // C5
        (10 * HALF, 18), // A4
        (11 * HALF, 16), // G4
        (12 * HALF, 14), // F4
        (13 * HALF, 13), // E4
        (14 * HALF, 11), // D4
        (15 * HALF, 6),  // A3
    ];

    let phrase_len = 8 * BARREL_STEPS_PER_BEAT; // 96 steps = 8 beats
    for rep in 0..2_usize {
        for &(offset, ch) in phrase {
            let step = rep * phrase_len + offset;
            if step < BARREL_N_STEPS && ch < BARREL_N_CHANNELS {
                p[step][ch] = true;
            }
        }
    }

    p
}

impl BarrelParams {
    pub fn reset_position(&mut self) {
        self.angle =
            std::f32::consts::TAU * (1.0 - 0.005 / BARREL_N_STEPS as f32);
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
        BARREL_CH_CHUTE => "Chute",
        BARREL_CH_DROP => "Drop",
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
        BARREL_CH_CHUTE => (51, 115, 230),  // blue – chute marble
        BARREL_CH_DROP => (242, 89, 38),    // orange – drop marble
        _ => (80, 200, 120),                // green – vibraphone
    }
}
