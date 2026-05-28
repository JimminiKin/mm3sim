use bevy::prelude::*;

use crate::components::hihat::HiHatTopCymbal;
use crate::resources::constants::*;
use crate::resources::hihat_params::{HiHatParams, HiHatState};
use crate::resources::programming_wheel_params::{ProgrammingWheelParams, WHEEL_CH_HIHAT_PEDAL};

/// Drives `HiHatState` from the piano-roll beat position:
/// closed while any HiHatPedal note spans `current_beat`, open otherwise.
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

/// Moves the top cymbal mesh to reflect the current open/closed state and hi-hat position.
pub fn update_hihat_visual(
    state: Res<HiHatState>,
    hihat_params: Res<HiHatParams>,
    mut top: Query<&mut Transform, With<HiHatTopCymbal>>,
) {
    if !state.is_changed() && !hihat_params.is_changed() {
        return;
    }
    let tilt = Quat::from_rotation_x(ARM_SPAWN_DEG.to_radians());
    let gap = if state.open { hihat_params.gap_open } else { hihat_params.gap_closed };
    let offset = tilt * Vec3::Y * (gap + HIHAT_HALF_HEIGHT * 2.0);
    for mut tf in &mut top {
        tf.translation = hihat_params.pos + offset;
        tf.rotation = tilt;
    }
}
