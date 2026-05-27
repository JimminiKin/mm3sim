use bevy::prelude::*;

/// Whether the hi-hat is currently open (long sustain) or closed (short tick).
/// Toggled each time a marble hits the `HiHatPedal` instrument.
#[derive(Resource)]
pub struct HiHatState {
    pub open: bool,
}

impl Default for HiHatState {
    fn default() -> Self {
        HiHatState { open: true }
    }
}
