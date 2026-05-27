use bevy::prelude::*;

use crate::components::hihat::HiHatTopCymbal;
use crate::resources::constants::*;
use crate::resources::hihat_params::HiHatState;
use crate::resources::programming_wheel_params::{ProgrammingWheelParams, WHEEL_CH_HIHAT_PEDAL};

/// Drives `HiHatState` from the piano-roll beat position:
/// closed while any HiHatPedal note spans `current_beat`, open otherwise.
/// Runs after `rotate_programming_wheel_system` updates `current_beat`.
pub fn sync_hihat_pedal_state(
    params: Res<ProgrammingWheelParams>,
    mut state: ResMut<HiHatState>,
) {
    let beat = params.current_beat;
    let closed = params.notes.iter().any(|n| {
        n.channel == WHEEL_CH_HIHAT_PEDAL && beat >= n.beat && beat < n.beat + n.length
    });
    let new_open = !closed;
    if state.open != new_open {
        state.open = new_open;
    }
}

/// Moves the top cymbal mesh up or down to reflect the current open/closed state.
/// Runs after `sync_hihat_pedal_state`.
pub fn update_hihat_visual(
    state: Res<HiHatState>,
    mut top: Query<&mut Transform, With<HiHatTopCymbal>>,
) {
    if !state.is_changed() {
        return;
    }
    let tilt = Quat::from_rotation_x(ARM_SPAWN_DEG.to_radians());
    let gap = if state.open { HIHAT_GAP_OPEN } else { HIHAT_GAP_CLOSED };
    let offset = tilt * Vec3::Y * (gap + HIHAT_HALF_HEIGHT * 2.0);
    for mut tf in &mut top {
        tf.translation = Vec3::new(HIHAT_X, HIHAT_Y, HIHAT_Z) + offset;
        tf.rotation = tilt;
    }
}
