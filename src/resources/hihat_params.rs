use bevy::prelude::*;

use crate::resources::constants::*;

/// Whether the hi-hat is currently open (long sustain) or closed (short tick).
#[derive(Resource)]
pub struct HiHatState {
    pub open: bool,
}

impl Default for HiHatState {
    fn default() -> Self {
        HiHatState { open: true }
    }
}

#[derive(Resource)]
pub struct HiHatParams {
    pub pos: Vec3,
    pub restitution: f32,
    pub friction: f32,
    pub gap_open: f32,
    pub gap_closed: f32,
    pub dirty: bool,
}

impl Default for HiHatParams {
    fn default() -> Self {
        Self {
            pos: Vec3::new(HIHAT_X, HIHAT_Y, HIHAT_Z),
            restitution: HIHAT_RESTITUTION,
            friction: HIHAT_FRICTION,
            gap_open: HIHAT_GAP_OPEN,
            gap_closed: HIHAT_GAP_CLOSED,
            dirty: false,
        }
    }
}
