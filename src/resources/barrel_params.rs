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
            pattern: vec![vec![false; BARREL_N_CHANNELS]; BARREL_N_STEPS],
            drag_paint_val: None,
            editor_open: false,
            show_pegs: true,
            pending_spawns: Vec::new(),
        }
    }
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
