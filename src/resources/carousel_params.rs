use bevy::prelude::*;

use crate::resources::constants::*;

#[derive(Resource)]
pub struct CarouselParams {
    pub pos: Vec3,
    /// Tilt of each instrument away from the arm axis (degrees).
    /// 0° = flat face points directly outward; 90° = face is parallel to arm.
    pub tilt_deg: f32,
    pub crash_restitution: f32,
    pub crash_friction: f32,
    pub cowbell_restitution: f32,
    pub cowbell_friction: f32,
    pub tamb_restitution: f32,
    pub tamb_friction: f32,
    pub wood_restitution: f32,
    pub wood_friction: f32,
    pub dirty: bool,
}

impl Default for CarouselParams {
    fn default() -> Self {
        Self {
            pos: Vec3::new(CAROUSEL_X, CAROUSEL_Y, CAROUSEL_Z),
            tilt_deg: CAROUSEL_TILT_DEG,
            crash_restitution: CAROUSEL_CRASH_RESTITUTION,
            crash_friction: CAROUSEL_CRASH_FRICTION,
            cowbell_restitution: CAROUSEL_COWBELL_RESTITUTION,
            cowbell_friction: CAROUSEL_COWBELL_FRICTION,
            tamb_restitution: CAROUSEL_TAMB_RESTITUTION,
            tamb_friction: CAROUSEL_TAMB_FRICTION,
            wood_restitution: CAROUSEL_WOOD_RESTITUTION,
            wood_friction: CAROUSEL_WOOD_FRICTION,
            dirty: false,
        }
    }
}

/// Tracks the carousel's rotation state across frames.
#[derive(Resource)]
pub struct CarouselState {
    /// Current carousel rotation angle around the X axis (radians).
    pub current_angle: f32,
    /// Target angle to animate toward (radians; may exceed 2π for smooth unwinding).
    pub target_angle: f32,
    pub is_animating: bool,
    /// Quarter-turns queued by the selector channel, consumed by the animation system.
    pub pending_advances: u32,
    /// Which slot (0–3) is currently at the top (0=crash, 1=cowbell, 2=tambourine, 3=woodblock).
    pub current_slot: u8,
}

impl Default for CarouselState {
    fn default() -> Self {
        Self {
            current_angle: 0.0,
            target_angle: 0.0,
            is_animating: false,
            pending_advances: 0,
            current_slot: 0,
        }
    }
}
