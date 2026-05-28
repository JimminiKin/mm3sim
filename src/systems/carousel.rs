use bevy::prelude::*;

use crate::components::carousel::{CarouselArm, CarouselBody, CarouselSlot, arm_transform, slot_transform};
use crate::resources::carousel_params::{CarouselParams, CarouselState};
use crate::resources::constants::*;

/// Advances the carousel angle toward `target_angle` at `CAROUSEL_ROTATION_SPEED` rad/s.
/// Consumes `pending_advances` to start new quarter-turn rotations.
pub fn animate_carousel_system(time: Res<Time>, mut state: ResMut<CarouselState>) {
    if state.pending_advances > 0 && !state.is_animating {
        state.target_angle += state.pending_advances as f32 * std::f32::consts::FRAC_PI_2;
        state.pending_advances = 0;
        state.is_animating = true;
    }

    if !state.is_animating {
        return;
    }

    let remaining = state.target_angle - state.current_angle;
    if remaining.abs() < 0.002 {
        state.current_angle = state.target_angle;
        state.is_animating = false;
        let normalized = state.current_angle.rem_euclid(std::f32::consts::TAU);
        state.current_slot =
            ((normalized / std::f32::consts::FRAC_PI_2).round() as u8) % 4;
        return;
    }

    let step = CAROUSEL_ROTATION_SPEED * time.delta_secs();
    state.current_angle += step.min(remaining.abs()) * remaining.signum();
}

/// Repositions and reorients each instrument and its arm to follow the current carousel angle.
pub fn update_carousel_instruments(
    state: Res<CarouselState>,
    params: Res<CarouselParams>,
    // Without filters prevent Bevy from aliasing mutable Transform access across queries.
    mut slots: Query<(&CarouselSlot, &mut Transform), Without<CarouselArm>>,
    mut arms: Query<(&CarouselArm, &mut Transform), Without<CarouselSlot>>,
) {
    if !state.is_changed() && !params.is_changed() {
        return;
    }
    for (slot, mut tf) in &mut slots {
        *tf = slot_transform(&params, slot.0, state.current_angle);
    }
    for (arm, mut tf) in &mut arms {
        *tf = arm_transform(&params, arm.0, state.current_angle);
    }
}

/// Keeps the central axis cylinder centred on `params.pos` when params are rebuilt.
pub fn update_carousel_body(
    params: Res<CarouselParams>,
    mut bodies: Query<&mut Transform, With<CarouselBody>>,
) {
    if !params.is_changed() {
        return;
    }
    for mut tf in &mut bodies {
        tf.translation = params.pos;
    }
}
