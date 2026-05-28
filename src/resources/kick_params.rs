use bevy::prelude::*;

use crate::resources::constants::*;

#[derive(Resource)]
pub struct KickParams {
    pub pos: Vec3,
    pub restitution: f32,
    pub friction: f32,
    pub rest_deg: f32,
    pub max_tilt_deg: f32,
    pub angular_damping: f32,
    pub cw_weight_ratio: f32,
    pub dirty: bool,
}

impl Default for KickParams {
    fn default() -> Self {
        Self {
            pos: Vec3::new(KICK_X, KICK_Y, KICK_Z),
            restitution: KICK_RESTITUTION,
            friction: KICK_FRICTION,
            rest_deg: KICK_REST_DEG,
            max_tilt_deg: KICK_MAX_TILT_DEG,
            angular_damping: KICK_ANGULAR_DAMPING,
            cw_weight_ratio: KICK_CW_WEIGHT_RATIO,
            dirty: false,
        }
    }
}
